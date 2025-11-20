use pinocchio::program_error::ProgramError;

use crate::{
    constants::{NUM_REWARDS, PROTOCOL_FEE_RATE_MUL_VALUE},
    math::{Q64_RESOLUTION, add_liquidity_delta, sqrt_price_from_tick_index},
    state::{AdaptiveFeeInfo, Tick, WhirlpoolRewardInfo},
};

#[derive(Debug)]
pub struct PostSwapUpdate {
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_fee: u64,
    pub next_liquidity: u128,
    pub next_tick_index: i32,
    pub next_sqrt_price: u128,
    pub next_fee_growth_global: u128,
    pub next_reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS],
    pub next_protocol_fee: u64,
    pub next_adaptive_fee_info: Option<AdaptiveFeeInfo>,
}

pub fn get_next_sqrt_prices(
    next_tick_index: i32,
    sqrt_price_limit: u128,
    a_to_b: bool,
) -> (u128, u128) {
    let next_tick_price = sqrt_price_from_tick_index(next_tick_index);
    let next_sqrt_price_limit = if a_to_b {
        sqrt_price_limit.max(next_tick_price)
    } else {
        sqrt_price_limit.min(next_tick_price)
    };
    (next_tick_price, next_sqrt_price_limit)
}

pub fn calculate_next_liquidity(
    tick: &Tick,
    a_to_b: bool,
    liquidity: u128,
) -> Result<u128, ProgramError> {
    // Use updated fee_growth for crossing tick
    // Use -liquidity_net if going left, +liquidity_net going right
    let signed_liquidity_net = if a_to_b {
        -tick.liquidity_net
    } else {
        tick.liquidity_net
    };

    // Update the global liquidity to reflect the new current tick
    let next_liquidity = add_liquidity_delta(liquidity, signed_liquidity_net)?;

    Ok(next_liquidity)
}

pub fn calculate_fees(
    fee_amount: u64,
    protocol_fee_rate: u16,
    curr_liquidity: u128,
    curr_protocol_fee: u64,
    curr_fee_growth_global_input: u128,
) -> (u64, u128) {
    let mut next_protocol_fee = curr_protocol_fee;
    let mut next_fee_growth_global_input = curr_fee_growth_global_input;
    let mut global_fee = fee_amount;
    if protocol_fee_rate > 0 {
        let delta = calculate_protocol_fee(global_fee, protocol_fee_rate);
        global_fee -= delta;
        next_protocol_fee = next_protocol_fee.wrapping_add(delta);
    }

    if curr_liquidity > 0 {
        next_fee_growth_global_input = next_fee_growth_global_input
            .wrapping_add(((global_fee as u128) << Q64_RESOLUTION) / curr_liquidity);
    }
    (next_protocol_fee, next_fee_growth_global_input)
}

fn calculate_protocol_fee(global_fee: u64, protocol_fee_rate: u16) -> u64 {
    ((global_fee as u128) * (protocol_fee_rate as u128) / PROTOCOL_FEE_RATE_MUL_VALUE)
        .try_into()
        .unwrap()
}
