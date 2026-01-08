use crate::{
    constants::{TICK_ARRAY_SIZE, TICK_ARRAY_SIZE_USIZE},
    error::OrcaError,
    state::*,
};
use arrayref::array_ref;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::Address;
use solana_program_error::ProgramError;

#[derive(BorshDeserialize, BorshSerialize, Clone, Default, Debug, PartialEq, Copy)]
pub struct DynamicTickData {
    pub liquidity_net: i128,   // 16
    pub liquidity_gross: u128, // 16

    // Q64.64
    pub fee_growth_outside_a: u128, // 16
    // Q64.64
    pub fee_growth_outside_b: u128, // 16

    // Array of Q64.64
    pub reward_growths_outside: [u128; 3], // 48 = 16 * 3
}

impl DynamicTickData {
    pub const LEN: usize = 112;
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Default, Debug, PartialEq, Copy)]
pub enum DynamicTick {
    #[default]
    Uninitialized,
    Initialized(DynamicTickData),
}

impl DynamicTick {
    pub const UNINITIALIZED_LEN: usize = 1;
    pub const INITIALIZED_LEN: usize = DynamicTickData::LEN + 1;
}

impl From<&TickUpdate> for DynamicTick {
    fn from(update: &TickUpdate) -> Self {
        if update.initialized {
            DynamicTick::Initialized(DynamicTickData {
                liquidity_net: update.liquidity_net,
                liquidity_gross: update.liquidity_gross,
                fee_growth_outside_a: update.fee_growth_outside_a,
                fee_growth_outside_b: update.fee_growth_outside_b,
                reward_growths_outside: update.reward_growths_outside,
            })
        } else {
            DynamicTick::Uninitialized
        }
    }
}

impl From<DynamicTick> for Tick {
    fn from(val: DynamicTick) -> Self {
        match val {
            DynamicTick::Uninitialized => Tick::default(),
            DynamicTick::Initialized(tick_data) => Tick {
                initialized: 1,
                liquidity_net: tick_data.liquidity_net,
                liquidity_gross: tick_data.liquidity_gross,
                fee_growth_outside_a: tick_data.fee_growth_outside_a,
                fee_growth_outside_b: tick_data.fee_growth_outside_b,
                reward_growths_outside: tick_data.reward_growths_outside,
            },
        }
    }
}

// This struct is never actually used anywhere.
// account attr is used to generate the definition in the IDL.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct DynamicTickArray {
    pub start_tick_index: i32, // 4 bytes
    pub whirlpool: Address,    // 32 bytes
    // 0: uninitialized, 1: initialized
    pub tick_bitmap: u128, // 16 bytes
    pub ticks: [DynamicTick; TICK_ARRAY_SIZE_USIZE],
}

impl DynamicTickArray {
    pub const DISCRIMINATOR: &[u8] = &[17, 216, 246, 142, 225, 199, 218, 56];

    pub const MIN_LEN: usize = DynamicTickArray::DISCRIMINATOR.len()
        + 4
        + 32
        + 16
        + DynamicTick::UNINITIALIZED_LEN * TICK_ARRAY_SIZE_USIZE;
    pub const MAX_LEN: usize = DynamicTickArray::DISCRIMINATOR.len()
        + 4
        + 32
        + 16
        + DynamicTick::INITIALIZED_LEN * TICK_ARRAY_SIZE_USIZE;
}

#[derive(Debug, Clone)]
pub struct DynamicTickArrayLoader([u8; DynamicTickArray::MAX_LEN]);

#[cfg(test)]
impl Default for DynamicTickArrayLoader {
    fn default() -> Self {
        Self([0; DynamicTickArray::MAX_LEN])
    }
}

impl DynamicTickArrayLoader {
    // Reimplement these functions from bytemuck::from_bytes_mut without
    // the size and alignment checks. If reading beyond the end of the underlying
    // data, the behavior is undefined.

    pub fn load(data: &[u8]) -> &DynamicTickArrayLoader {
        unsafe { &*(data.as_ptr() as *const DynamicTickArrayLoader) }
    }

    pub fn load_mut(data: &mut [u8]) -> &mut DynamicTickArrayLoader {
        unsafe { &mut *(data.as_mut_ptr() as *mut DynamicTickArrayLoader) }
    }

    // Data layout:
    // 4 bytes for start_tick_index i32
    // 32 bytes for whirlpool pubkey
    // 88 to 9944 bytes for tick data

    const START_TICK_INDEX_OFFSET: usize = 0;
    const WHIRLPOOL_OFFSET: usize = Self::START_TICK_INDEX_OFFSET + 4;
    const TICK_BITMAP_OFFSET: usize = Self::WHIRLPOOL_OFFSET + 32;
    const TICK_DATA_OFFSET: usize = Self::TICK_BITMAP_OFFSET + 16;

    // pub fn initialize(
    //     &mut self,
    //     whirlpool: &Account<Whirlpool>,
    //     start_tick_index: i32,
    // ) -> Result<(), ProgramError> {
    //     if !Tick::check_is_valid_start_tick(start_tick_index, whirlpool.tick_spacing) {
    //         return Err(ErrorCode::InvalidStartTick.into());
    //     }

    //     self.0[Self::START_TICK_INDEX_OFFSET..Self::START_TICK_INDEX_OFFSET + 4]
    //         .copy_from_slice(&start_tick_index.to_le_bytes());
    //     self.0[Self::WHIRLPOOL_OFFSET..Self::WHIRLPOOL_OFFSET + 32]
    //         .copy_from_slice(&whirlpool.key().to_bytes());
    //     Ok(())
    // }

    fn tick_data(&self) -> &[u8] {
        &self.0[Self::TICK_DATA_OFFSET..]
    }

    fn tick_data_mut(&mut self) -> &mut [u8] {
        &mut self.0[Self::TICK_DATA_OFFSET..]
    }
}

impl TickArrayType for DynamicTickArrayLoader {
    fn is_variable_size(&self) -> bool {
        true
    }

    fn start_tick_index(&self) -> i32 {
        i32::from_le_bytes(*array_ref![self.0, Self::START_TICK_INDEX_OFFSET, 4])
    }

    fn whirlpool(&self) -> Address {
        Address::new_from_array(*array_ref![self.0, Self::WHIRLPOOL_OFFSET, 32])
    }

    fn get_next_init_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>, ProgramError> {
        if !self.in_search_range(tick_index, tick_spacing, !a_to_b) {
            return Err(OrcaError::InvalidTickArraySequence.into());
        }

        let mut curr_offset = match self.tick_offset(tick_index, tick_spacing) {
            Ok(value) => value as i32,
            Err(e) => return Err(e),
        };

        // For a_to_b searches, the search moves to the left. The next possible init-tick can be the 1st tick in the current offset
        // For b_to_a searches, the search moves to the right. The next possible init-tick cannot be within the current offset
        if !a_to_b {
            curr_offset += 1;
        }

        let tick_bitmap = self.tick_bitmap();
        while (0..TICK_ARRAY_SIZE).contains(&curr_offset) {
            let initialized = Self::is_initialized_tick(&tick_bitmap, curr_offset as isize);
            if initialized {
                return Ok(Some(
                    (curr_offset * tick_spacing as i32) + self.start_tick_index(),
                ));
            }

            curr_offset = if a_to_b {
                curr_offset - 1
            } else {
                curr_offset + 1
            };
        }

        Ok(None)
    }

    fn get_tick(&self, tick_index: i32, tick_spacing: u16) -> Result<Tick, ProgramError> {
        if !self.check_in_array_bounds(tick_index, tick_spacing)
            || !Tick::check_is_usable_tick(tick_index, tick_spacing)
        {
            return Err(OrcaError::TickNotFound.into());
        }
        let tick_offset = self.tick_offset(tick_index, tick_spacing)?;
        let byte_offset = self.byte_offset(tick_offset)?;
        let ticks_data = self.tick_data();
        let mut tick_data = &ticks_data[byte_offset..byte_offset + DynamicTick::INITIALIZED_LEN];
        let tick =
            DynamicTick::deserialize(&mut tick_data).map_err(|_| ProgramError::BorshIoError)?;
        Ok(tick.into())
    }

    fn update_tick(
        &mut self,
        tick_index: i32,
        tick_spacing: u16,
        update: &TickUpdate,
    ) -> Result<(), ProgramError> {
        if !self.check_in_array_bounds(tick_index, tick_spacing)
            || !Tick::check_is_usable_tick(tick_index, tick_spacing)
        {
            return Err(OrcaError::TickNotFound.into());
        }
        let tick_offset = self.tick_offset(tick_index, tick_spacing)?;
        let byte_offset = self.byte_offset(tick_offset)?;
        let data = self.tick_data();
        let mut tick_data = &data[byte_offset..byte_offset + DynamicTick::INITIALIZED_LEN];
        let tick: Tick = DynamicTick::deserialize(&mut tick_data)
            .map_err(|_| ProgramError::BorshIoError)?
            .into();

        // If the tick needs to be initialized, we need to right-shift everything after byte_offset by DynamicTickData::LEN
        if !tick.initialized == 1 && update.initialized {
            let data_mut = self.tick_data_mut();
            let shift_data = &mut data_mut[byte_offset..];
            shift_data.rotate_right(DynamicTickData::LEN);

            // sync bitmap
            self.update_tick_bitmap(tick_offset, true);
        }

        // If the tick needs to be uninitialized, we need to left-shift everything after byte_offset by DynamicTickData::LEN
        if tick.initialized == 1 && !update.initialized {
            let data_mut = self.tick_data_mut();
            let shift_data = &mut data_mut[byte_offset..];
            shift_data.rotate_left(DynamicTickData::LEN);

            // sync bitmap
            self.update_tick_bitmap(tick_offset, false);
        }

        // Update the tick data at byte_offset
        let tick_data_len = if update.initialized {
            DynamicTick::INITIALIZED_LEN
        } else {
            DynamicTick::UNINITIALIZED_LEN
        };

        let data_mut = self.tick_data_mut();
        let mut tick_data = &mut data_mut[byte_offset..byte_offset + tick_data_len];
        DynamicTick::from(update)
            .serialize(&mut tick_data)
            .map_err(|_| ProgramError::BorshIoError)?;

        Ok(())
    }
}

impl DynamicTickArrayLoader {
    fn byte_offset(&self, tick_offset: isize) -> Result<usize, ProgramError> {
        if tick_offset < 0 {
            return Err(OrcaError::TickNotFound.into());
        }

        let tick_bitmap = self.tick_bitmap();
        let mask = (1u128 << tick_offset) - 1;
        let initialized_ticks = (tick_bitmap & mask).count_ones() as usize;
        let uninitialized_ticks = tick_offset as usize - initialized_ticks;

        let offset = initialized_ticks * DynamicTick::INITIALIZED_LEN
            + uninitialized_ticks * DynamicTick::UNINITIALIZED_LEN;
        Ok(offset)
    }

    fn tick_bitmap(&self) -> u128 {
        u128::from_le_bytes(*array_ref![self.0, Self::TICK_BITMAP_OFFSET, 16])
    }

    fn update_tick_bitmap(&mut self, tick_offset: isize, initialized: bool) {
        let mut tick_bitmap = self.tick_bitmap();
        if initialized {
            tick_bitmap |= 1 << tick_offset;
        } else {
            tick_bitmap &= !(1 << tick_offset);
        }
        self.0[Self::TICK_BITMAP_OFFSET..Self::TICK_BITMAP_OFFSET + 16]
            .copy_from_slice(&tick_bitmap.to_le_bytes());
    }

    #[inline(always)]
    fn is_initialized_tick(tick_bitmap: &u128, tick_offset: isize) -> bool {
        (*tick_bitmap & (1 << tick_offset)) != 0
    }
}
