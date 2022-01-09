//! Instruction
use crate::error::VestingError;

use solana_program::{
    msg,
    program_error::ProgramError,
    pubkey::Pubkey
};

use std::convert::TryInto;
use std::mem::size_of;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
/// Setup one schedule for release money to target account according to the order in schedule array.
pub struct Schedule {
    ///Release time: unix timestamp
    pub release_time: u64,
    ///Amout for this release
    pub amount: u64,
}

///Fixed schedule length for parsing multiple release time in input data
pub const SCHEDULE_LENGTH: usize = 16;

///Get data from records
pub fn get_data(records: &str, name: &str){
    let mut pos = records.find(name);
    msg!("{:?}",pos)
    let mut c : Vec<_> = records.match_indices("|").collect();
    msg!("{:?}",c)
}
///Vesting instruction, including derived vesting address and the number of schedules
#[derive(Clone, Debug, PartialEq)]
pub enum VestingInstruction {
    ///Initialize a program account for vesting
    Initialize {
        ///For generating a vesting account
        derived_vesting_address: [u8; 32],
        ///The number of time slots for releasing money
        number_of_schedules: u32,
    },

    ///Create vesting plan
    Create {
        ///For generating a vesting address
        derived_vesting_address: [u8; 32],
        ///Mint address
        mint_address: Pubkey,
        ///Destination address
        destination_token_address: Pubkey,
        ///Release schedules
        schedules: Vec<Schedule>,
    },

    ///Unlock money
    Unlock { 
            ///Unlock money
            derived_vesting_address: [u8; 32] 
        },

    ///Change the destination account address
    ChangeDestination { 
        ///Change the destination account address
        derived_vesting_address: [u8; 32]
     },
}

impl VestingInstruction {
    ///Unpack slice
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use VestingError::BadInstruction;
        let (&tag, rest) = input.split_first().ok_or(BadInstruction)?;
        Ok(match tag {
            0 => {
                let derived_vesting_address: [u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok())
                    .unwrap();

                let number_of_schedules = rest
                    .get(32..36)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u32::from_le_bytes)
                    .ok_or(BadInstruction)?;
                

                let records = rest.get(36..rest.len()) //
                .and_then(|slice| slice.try_into().ok())
                .map(std::str::from_utf8)
                .ok_or(BadInstruction)?;

                match records {
                    Ok(value) => {
                        get_data(&value,"target_account:");
                    },
                    Err(error) => msg!("{}", error),
                }

                Self::Initialize {
                    derived_vesting_address,
                    number_of_schedules,
                }
            }
            1 => {
                let derived_vesting_address: [u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok())
                    .unwrap();

                let mint_address = rest
                    .get(32..64)
                    .and_then(|slice| slice.try_into().ok())
                    .map(Pubkey::new)
                    .ok_or(BadInstruction)?;

                let destination_token_address = rest
                    .get(64..96)
                    .and_then(|slice| slice.try_into().ok())
                    .map(Pubkey::new)
                    .ok_or(BadInstruction)?;

                let number_of_schedules = rest[96..].len() / SCHEDULE_LENGTH;
                let mut schedules: Vec<Schedule> = Vec::with_capacity(number_of_schedules);
                let mut offset = 96;
                for _ in 0..number_of_schedules {
                    let release_time = rest
                        .get(offset..offset + 8)
                        .and_then(|slice| slice.try_into().ok())
                        .map(u64::from_le_bytes)
                        .ok_or(BadInstruction)?;

                    let amount = rest
                        .get(offset + 8..offset + 16)
                        .and_then(|slice| slice.try_into().ok())
                        .map(u64::from_le_bytes)
                        .ok_or(BadInstruction)?;

                    offset += SCHEDULE_LENGTH;
                    schedules.push(Schedule {
                        release_time,
                        amount,
                    })
                }
                Self::Create {
                    derived_vesting_address,
                    mint_address,
                    destination_token_address,
                    schedules,
                }
            }
            2 | 3 => {
                let derived_vesting_address: [u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok())
                    .unwrap();
                match tag {
                    2 => Self::Unlock { derived_vesting_address },
                    _ => Self::ChangeDestination { derived_vesting_address },
                }
            }
            _ => {
                msg!("Error: please provide the proper tag");
                return Err(BadInstruction.into());
            }
        })
    }
    ///Pack data
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Initialize {
                derived_vesting_address,
                number_of_schedules,
            } => {
                buf.push(0);
                buf.extend_from_slice(&derived_vesting_address);
                buf.extend_from_slice(&number_of_schedules.to_le_bytes())
            }
            Self::Create {
                derived_vesting_address,
                mint_address,
                destination_token_address,
                schedules,
            } => {
                buf.push(1);
                buf.extend_from_slice(derived_vesting_address);
                buf.extend_from_slice(&mint_address.to_bytes());
                buf.extend_from_slice(&destination_token_address.to_bytes());
                for schedule in schedules.iter() {
                    buf.extend_from_slice(&schedule.release_time.to_le_bytes());
                    buf.extend_from_slice(&schedule.amount.to_le_bytes());
                }
            }
            &Self::Unlock { derived_vesting_address } => {
                buf.push(2);
                buf.extend_from_slice(&derived_vesting_address);
            }
            &Self::ChangeDestination { derived_vesting_address } => {
                buf.push(3);
                buf.extend_from_slice(&derived_vesting_address);
            }
        };
        buf
    }
}
