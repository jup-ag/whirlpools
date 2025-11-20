use bytemuck::{Pod, Zeroable};
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    constants::{
        MAX_REFERENCE_AGE, REDUCTION_FACTOR_DENOMINATOR, VOLATILITY_ACCUMULATOR_SCALE_FACTOR,
    },
    math::{Q64_RESOLUTION, U256Muldiv, increasing_price_order, sqrt_price_from_tick_index},
};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, packed)]
pub struct Oracle {
    pub whirlpool: Pubkey,
    pub trade_enable_timestamp: u64,
    pub adaptive_fee_constants: AdaptiveFeeConstants,
    pub adaptive_fee_variables: AdaptiveFeeVariables,
    // Reserved for future use
    pub reserved: [u8; 128],
}

impl Oracle {
    pub const DISCRIMINATOR: &'static [u8] = &[139, 194, 131, 179, 140, 179, 229, 244];
}

#[derive(Debug, Default, Clone)]
pub struct AdaptiveFeeInfo {
    pub constants: AdaptiveFeeConstants,
    pub variables: AdaptiveFeeVariables,
}

#[derive(Pod, Zeroable)]
#[repr(C, packed)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct AdaptiveFeeConstants {
    // Period determine high frequency trading time window
    // The unit of time is "seconds" and is applied to the chain's block time
    pub filter_period: u16,
    // Period determine when the adaptive fee start decrease
    // The unit of time is "seconds" and is applied to the chain's block time
    pub decay_period: u16,
    // Adaptive fee rate decrement rate
    pub reduction_factor: u16,
    // Used to scale the adaptive fee component
    pub adaptive_fee_control_factor: u32,
    // Maximum number of ticks crossed can be accumulated
    // Used to cap adaptive fee rate
    pub max_volatility_accumulator: u32,
    // Tick group index is defined as floor(tick_index / tick_group_size)
    pub tick_group_size: u16,
    // Major swap threshold in tick
    pub major_swap_threshold_ticks: u16,
    // Reserved for future use
    pub reserved: [u8; 16],
}

#[derive(Pod, Zeroable)]
#[repr(C, packed)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct AdaptiveFeeVariables {
    // Last timestamp (block time) when volatility_reference and tick_group_index_reference were updated
    pub last_reference_update_timestamp: u64,
    // Last timestamp (block time) when major swap was executed
    pub last_major_swap_timestamp: u64,
    // Volatility reference is decayed volatility accumulator
    pub volatility_reference: u32,
    // Active tick group index of last swap
    pub tick_group_index_reference: i32,
    // Volatility accumulator measure the number of tick group crossed since reference tick group index (scaled)
    pub volatility_accumulator: u32,
    // Reserved for future use
    pub reserved: [u8; 16],
}

impl AdaptiveFeeVariables {
    pub const LEN: usize = 8 + 8 + 4 + 4 + 4 + 16;

    pub fn update_volatility_accumulator(
        &mut self,
        tick_group_index: i32,
        adaptive_fee_constants: &AdaptiveFeeConstants,
    ) -> Result<(), ProgramError> {
        let index_delta = (self.tick_group_index_reference - tick_group_index).unsigned_abs();
        let volatility_accumulator = u64::from(self.volatility_reference)
            + u64::from(index_delta) * u64::from(VOLATILITY_ACCUMULATOR_SCALE_FACTOR);

        self.volatility_accumulator = core::cmp::min(
            volatility_accumulator,
            u64::from(adaptive_fee_constants.max_volatility_accumulator),
        ) as u32;

        Ok(())
    }

    pub fn update_reference(
        &mut self,
        tick_group_index: i32,
        current_timestamp: u64,
        adaptive_fee_constants: &AdaptiveFeeConstants,
    ) -> Result<(), ProgramError> {
        let max_timestamp = self
            .last_reference_update_timestamp
            .max(self.last_major_swap_timestamp);
        // if current_timestamp < max_timestamp {
        //     return Err(ErrorCode::InvalidTimestamp.into());
        // }

        let reference_age = current_timestamp.saturating_sub(self.last_reference_update_timestamp);
        if reference_age > MAX_REFERENCE_AGE {
            // The references are too old, so reset them
            self.tick_group_index_reference = tick_group_index;
            self.volatility_reference = 0;
            self.last_reference_update_timestamp = current_timestamp;
            return Ok(());
        }

        let elapsed = current_timestamp.saturating_sub(max_timestamp);
        if elapsed < adaptive_fee_constants.filter_period as u64 {
            // high frequency trade
            // no change
        } else if elapsed < adaptive_fee_constants.decay_period as u64 {
            // NOT high frequency trade
            self.tick_group_index_reference = tick_group_index;
            self.volatility_reference = (u64::from(self.volatility_accumulator)
                * u64::from(adaptive_fee_constants.reduction_factor)
                / u64::from(REDUCTION_FACTOR_DENOMINATOR))
                as u32;
            self.last_reference_update_timestamp = current_timestamp;
        } else {
            // Out of decay time window
            self.tick_group_index_reference = tick_group_index;
            self.volatility_reference = 0;
            self.last_reference_update_timestamp = current_timestamp;
        }

        Ok(())
    }

    pub fn update_major_swap_timestamp(
        &mut self,
        pre_sqrt_price: u128,
        post_sqrt_price: u128,
        current_timestamp: u64,
        adaptive_fee_constants: &AdaptiveFeeConstants,
    ) -> Result<(), ProgramError> {
        if Self::is_major_swap(
            pre_sqrt_price,
            post_sqrt_price,
            adaptive_fee_constants.major_swap_threshold_ticks,
        )? {
            self.last_major_swap_timestamp = current_timestamp;
        }
        Ok(())
    }

    // Determine whether the difference between pre_sqrt_price and post_sqrt_price is equivalent to major_swap_threshold_ticks or more
    // Note: The error of less than 0.00000003% due to integer arithmetic of sqrt_price is acceptable
    fn is_major_swap(
        pre_sqrt_price: u128,
        post_sqrt_price: u128,
        major_swap_threshold_ticks: u16,
    ) -> Result<bool, ProgramError> {
        let (smaller_sqrt_price, larger_sqrt_price) =
            increasing_price_order(pre_sqrt_price, post_sqrt_price);

        // major_swap_sqrt_price_target
        //   = smaller_sqrt_price * sqrt(pow(1.0001, major_swap_threshold_ticks))
        //   = smaller_sqrt_price * sqrt_price_from_tick_index(major_swap_threshold_ticks) >> Q64_RESOLUTION
        //
        // Note: The following two are theoretically equal, but there is an integer arithmetic error.
        //       However, the error impact is less than 0.00000003% in sqrt price (x64) and is small enough.
        //       - sqrt_price_from_tick_index(a) * sqrt_price_from_tick_index(b) >> Q64_RESOLUTION   (mathematically, sqrt(pow(1.0001, a)) * sqrt(pow(1.0001, b)) = sqrt(pow(1.0001, a + b)))
        //       - sqrt_price_from_tick_index(a + b)                                                 (mathematically, sqrt(pow(1.0001, a + b)))
        let major_swap_sqrt_price_factor =
            sqrt_price_from_tick_index(major_swap_threshold_ticks as i32);
        let major_swap_sqrt_price_target = U256Muldiv::new(0, smaller_sqrt_price)
            .mul(U256Muldiv::new(0, major_swap_sqrt_price_factor))
            .shift_right(Q64_RESOLUTION as u32)
            .try_into_u128()?;

        Ok(larger_sqrt_price >= major_swap_sqrt_price_target)
    }
}
