//!Entrypoint
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};


use crate::{error::VestingError, processer::Processer};

entrypoint!(process_instruction);

///Process instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Vesting Entrypoint");
    if let Err(error) = Processer::process_instruction(program_id, accounts, instruction_data) {
        error.print::<VestingError>();
        return Err(error);
    }
    Ok(())
}
