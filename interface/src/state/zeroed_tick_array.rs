use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::OrcaError;

use super::{Tick, TickArrayType, TickUpdate};

pub struct ZeroedTickArray {
    pub start_tick_index: i32,
    zeroed_tick: Tick,
}

impl ZeroedTickArray {
    pub fn new(start_tick_index: i32) -> Self {
        ZeroedTickArray {
            start_tick_index,
            zeroed_tick: Tick::default(),
        }
    }
}

impl TickArrayType for ZeroedTickArray {
    fn is_variable_size(&self) -> bool {
        false
    }

    fn start_tick_index(&self) -> i32 {
        self.start_tick_index
    }

    fn whirlpool(&self) -> Pubkey {
        // Never actually used
        unreachable!()
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

        self.tick_offset(tick_index, tick_spacing)?;

        // no initialized tick
        Ok(None)
    }

    fn get_tick(&self, tick_index: i32, tick_spacing: u16) -> Result<Tick, ProgramError> {
        if !self.check_in_array_bounds(tick_index, tick_spacing)
            || !Tick::check_is_usable_tick(tick_index, tick_spacing)
        {
            return Err(OrcaError::TickNotFound.into());
        }
        let offset = self.tick_offset(tick_index, tick_spacing)?;
        if offset < 0 {
            return Err(OrcaError::TickNotFound.into());
        }

        // always return the zeroed tick
        Ok(self.zeroed_tick)
    }

    fn update_tick(
        &mut self,
        _tick_index: i32,
        _tick_spacing: u16,
        _update: &TickUpdate,
    ) -> Result<(), ProgramError> {
        panic!("ZeroedTickArray must not be updated");
    }
}
