//! Error Display
use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

///Vesting Error
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum VestingError {
    #[error("Bad Instruction")]
    ///Malformed istruction error
    BadInstruction
}

impl From<VestingError> for ProgramError {
    fn from(e: VestingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for VestingError {
    fn type_of() -> &'static str {
        "VestingError"
    }
}