use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::Address;
use solana_program_error::ProgramError;

use crate::{constants::MAX_PROTOCOL_FEE_RATE, error::OrcaError};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct WhirlpoolsConfig {
    pub fee_authority: Address,
    pub collect_protocol_fees_authority: Address,
    pub reward_emissions_super_authority: Address,

    pub default_protocol_fee_rate: u16,
}

impl WhirlpoolsConfig {
    pub const LEN: usize = 8 + 96 + 4;

    pub fn update_fee_authority(&mut self, fee_authority: Address) {
        self.fee_authority = fee_authority;
    }

    pub fn update_collect_protocol_fees_authority(
        &mut self,
        collect_protocol_fees_authority: Address,
    ) {
        self.collect_protocol_fees_authority = collect_protocol_fees_authority;
    }

    pub fn initialize(
        &mut self,
        fee_authority: Address,
        collect_protocol_fees_authority: Address,
        reward_emissions_super_authority: Address,
        default_protocol_fee_rate: u16,
    ) -> Result<(), ProgramError> {
        self.fee_authority = fee_authority;
        self.collect_protocol_fees_authority = collect_protocol_fees_authority;
        self.reward_emissions_super_authority = reward_emissions_super_authority;
        self.update_default_protocol_fee_rate(default_protocol_fee_rate)?;

        Ok(())
    }

    pub fn update_reward_emissions_super_authority(
        &mut self,
        reward_emissions_super_authority: Address,
    ) {
        self.reward_emissions_super_authority = reward_emissions_super_authority;
    }

    pub fn update_default_protocol_fee_rate(
        &mut self,
        default_protocol_fee_rate: u16,
    ) -> Result<(), ProgramError> {
        if default_protocol_fee_rate > MAX_PROTOCOL_FEE_RATE {
            return Err(OrcaError::ProtocolFeeRateMaxExceeded.into());
        }
        self.default_protocol_fee_rate = default_protocol_fee_rate;

        Ok(())
    }
}
