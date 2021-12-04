//! Data state
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::convert::TryInto;


#[derive(Debug, PartialEq)]
///Vesting schedule body containing release plan
pub struct VestingSchedule {
    ///The date of defreezing 
    pub release_time: u64,
    ///The amount for releasing
    pub amount: u64,
}

#[derive(Debug, PartialEq)]
///Schedule metadata
pub struct VestingScheduleInfo {
    ///Destination address for receiving money
    pub destination_address: Pubkey,
    ///Mint address of creating a schedule plan/contract
    pub mint_address: Pubkey,
    ///The schedule initlized state
    pub is_initialized: bool,
}

impl Sealed for VestingScheduleInfo {}

impl Pack for VestingScheduleInfo {
    const LEN: usize = 65;
    
    ///pack addresses to slice
    fn pack_into_slice(&self, target: &mut [u8]) {
        let destination_address_bytes = self.destination_address.to_bytes();
        let mint_address_bytes = self.mint_address.to_bytes();
        for i in 0..32 {
            target[i] = destination_address_bytes[i];
        }

        for i in 32..64 {
            target[i] = mint_address_bytes[i - 32];
        }

        target[64] = self.is_initialized as u8;
    }

    ///unpack addresses from slice
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < 65 {
            return Err(ProgramError::InvalidAccountData)
        }
        let destination_address = Pubkey::new(&src[..32]);
        let mint_address = Pubkey::new(&src[32..64]);
        let is_initialized = src[64] == 1;
        Ok(Self {
            destination_address,
            mint_address,
            is_initialized,
        })
    }
}

impl Sealed for VestingSchedule {}

impl Pack for VestingSchedule {
    const LEN: usize = 16;

    ///pack schedule to slice
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let release_time_bytes = self.release_time.to_le_bytes();
        let amount_bytes = self.amount.to_le_bytes();
        for i in 0..8 {
            dst[i] = release_time_bytes[i];
        }

        for i in 8..16 {
            dst[i] = amount_bytes[i - 8];
        }
    }

    ///unpack addresses from slice
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < 16 {
            return Err(ProgramError::InvalidAccountData)
        }
        let release_time = u64::from_le_bytes(src[0..8].try_into().unwrap());
        let amount = u64::from_le_bytes(src[8..16].try_into().unwrap());
        Ok(Self {
            release_time,
            amount,
        })
    }
}

impl IsInitialized for VestingScheduleInfo {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

///unpack schedule from slice
pub fn unpack_schedules(input: &[u8]) -> Result<Vec<VestingSchedule>, ProgramError> {
    let number_of_schedules = input.len() / VestingSchedule::LEN;
    let mut output: Vec<VestingSchedule> = Vec::with_capacity(number_of_schedules);
    let mut offset = 0;
    for _ in 0..number_of_schedules {
        output.push(VestingSchedule::unpack_from_slice(
            &input[offset..offset + VestingSchedule::LEN],
        )?);
        offset += VestingSchedule::LEN;
    }
    Ok(output)
}

///Pack schedule into slice
pub fn pack_schedules_into_slice(schedules: Vec<VestingSchedule>, target: &mut [u8]) {
    let mut offset = 0;
    for s in schedules.iter() {
        s.pack_into_slice(&mut target[offset..]);
        offset += VestingSchedule::LEN;
    }
}
