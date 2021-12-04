//! Process data state
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::PrintProgramError,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::{clock::Clock, Sysvar},
    msg,
};
use num_traits::FromPrimitive;
use spl_token::{instruction::transfer, state::Account};

use crate::{
    error::VestingError,
    instruction::{Schedule, VestingInstruction, SCHEDULE_LENGTH},
    state::{pack_schedules_into_slice, unpack_schedules, VestingSchedule, VestingScheduleInfo},
};

///Processer struct for process instruction and data state
pub struct Processer {}

impl Processer {
    ///Initalize the program
    pub fn process_init(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        derived_vesting_address: [u8; 32],
        schedules: u32
    ) -> ProgramResult {
     
        let accounts_iter = &mut accounts.iter();
        let system_program_account = next_account_info(accounts_iter)?;
        let rent_sysvar_account = next_account_info(accounts_iter)?;
        let payer = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let rent = Rent::from_account_info(rent_sysvar_account)?;
        let vesting_account_key = Pubkey::create_program_address(&[&derived_vesting_address], &program_id).unwrap();
        if vesting_account_key != *vesting_account.key {
            msg!("Error: please confirm your vesting account and key are correct.");
            return Err(ProgramError::InvalidArgument);
        }
        msg!("Processing Initialization");
        let state_size = (schedules as usize) * VestingSchedule::LEN + VestingScheduleInfo::LEN;

        msg!("Processing Initialization: init_vesting_account.");
        let init_vesting_account = create_account(
            &payer.key,
            &vesting_account_key,
            rent.minimum_balance(state_size),
            state_size as u64,
            &program_id,
        );

        msg!("Processing Initialization: invoke_signed.");
        invoke_signed(
            &init_vesting_account,
            &[
                system_program_account.clone(),
                payer.clone(),
                vesting_account.clone(),
            ],
            &[&[&derived_vesting_address]],
        )?;
        msg!("Processing Initialization: return ok.");
        Ok(())
    }

    ///Create vesting plan
    pub fn process_create(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        derived_vesting_address: [u8; 32],
        mint_address: &Pubkey,
        destination_token_address: &Pubkey,
        schedules: Vec<Schedule>,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let spl_token_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let source_token_account_owner = next_account_info(accounts_iter)?;
        let source_token_account = next_account_info(accounts_iter)?;

        msg!("Check vesting account key and vesting account");
        let vesting_account_key = Pubkey::create_program_address(&[&derived_vesting_address], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Error: please confirm your vesting account is correct.");
            return Err(ProgramError::InvalidArgument);
        }

        msg!("Check out vesting account and program_id");
        if *vesting_account.owner != *program_id {
            msg!("Error: Program must own vesting account");
            return Err(ProgramError::InvalidArgument);
        }

        if !source_token_account_owner.is_signer {
            msg!("Error: Source token account owner must be a signer.");
            return Err(ProgramError::InvalidArgument);
        }

        let is_initialized =
            vesting_account.try_borrow_data()?[VestingScheduleInfo::LEN - 1] == 1;

        if is_initialized {
            msg!("Error: Vesting contract exsits!");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("Error: The vesting token account is not the invalid vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        if vesting_token_account_data.delegate.is_some() {
            msg!("Error: The vesting token account should not be delegated.");
            return Err(ProgramError::InvalidAccountData);
        }

        if vesting_token_account_data.close_authority.is_some() {
            msg!("Error: The vesting token account must not have a close authority.");
            return Err(ProgramError::InvalidAccountData);
        }

        let state_info = VestingScheduleInfo {
            destination_address: *destination_token_address,
            mint_address: *mint_address,
            is_initialized: true,
        };

        let mut data = vesting_account.data.borrow_mut();
        if data.len() != VestingScheduleInfo::LEN + schedules.len() * VestingSchedule::LEN {
            return Err(ProgramError::InvalidAccountData)
        }
        state_info.pack_into_slice(&mut data);

        let mut offset = VestingScheduleInfo::LEN;
        let mut total_amount: u64 = 0;

        for s in schedules.iter() {
            let state_schedule = VestingSchedule {
                release_time: s.release_time,
                amount: s.amount,
            };
            state_schedule.pack_into_slice(&mut data[offset..]);
            let delta = total_amount.checked_add(s.amount);
            match delta {
                Some(n) => total_amount = n,
                None => return Err(ProgramError::InvalidInstructionData ), 
            }
            offset += SCHEDULE_LENGTH;
        }
        
        if Account::unpack(&source_token_account.data.borrow())?.amount < total_amount {
            msg!("The source token account has insufficient funds.");
            return Err(ProgramError::InsufficientFunds)
        };

        let transfer_tokens_to_vesting_account = transfer(
            spl_token_account.key,
            source_token_account.key,
            vesting_token_account.key,
            source_token_account_owner.key,
            &[],
            total_amount,
        )?;

        invoke(
            &transfer_tokens_to_vesting_account,
            &[
                source_token_account.clone(),
                vesting_token_account.clone(),
                spl_token_account.clone(),
                source_token_account_owner.clone(),
            ],
        )?;
        Ok(())
    }

    ///Check data to decide whether the destination account can unlock the money or not.
    pub fn process_unlock(
        program_id: &Pubkey,
        _accounts: &[AccountInfo],
        derived_vesting_address: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut _accounts.iter();

        let spl_token_account = next_account_info(accounts_iter)?;
        let clock_sysvar_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let destination_token_account = next_account_info(accounts_iter)?;

        let vesting_account_key = Pubkey::create_program_address(&[&derived_vesting_address], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Error: please check the vesting account key.");
            return Err(ProgramError::InvalidArgument);
        }

        if spl_token_account.key != &spl_token::id() {
            msg!("Error: please check provided spl token program account.");
            return Err(ProgramError::InvalidArgument)
        }

        let packed_state = &vesting_account.data;
        let info_state =
            VestingScheduleInfo::unpack(&packed_state.borrow()[..VestingScheduleInfo::LEN])?;

        if info_state.destination_address != *destination_token_account.key {
            msg!("Error: the contract destination account does not matched provided account.");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("Error: the vesting token account is not owned by the valid vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        let clock = Clock::from_account_info(&clock_sysvar_account)?;
        let mut total_amount_to_transfer = 0;

        let mut schedules = unpack_schedules(&packed_state.borrow()[VestingScheduleInfo::LEN..])?;
        for s in schedules.iter_mut() {
            if clock.unix_timestamp as u64 >= s.release_time {
                total_amount_to_transfer += s.amount;
                s.amount = 0;
            }
        }
        if total_amount_to_transfer == 0 {
            msg!("Error: Please wait for the arrival of the release time.");
            return Err(ProgramError::InvalidArgument);
        }

        let transfer_tokens_from_vesting_account = transfer(
            &spl_token_account.key,
            &vesting_token_account.key,
            destination_token_account.key,
            &vesting_account_key,
            &[],
            total_amount_to_transfer,
        )?;

        invoke_signed(
            &transfer_tokens_from_vesting_account,
            &[
                spl_token_account.clone(),
                vesting_token_account.clone(),
                destination_token_account.clone(),
                vesting_account.clone(),
            ],
            &[&[&derived_vesting_address]],
        )?;

        pack_schedules_into_slice(
            schedules,
            &mut packed_state.borrow_mut()[VestingScheduleInfo::LEN..],
        );

        Ok(())
    }

    ///Changing the destination account address
    pub fn process_change_destination(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        derived_vesting_address: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let vesting_account = next_account_info(accounts_iter)?;
        let destination_token_account = next_account_info(accounts_iter)?;
        let destination_token_account_owner = next_account_info(accounts_iter)?;
        let new_destination_token_account = next_account_info(accounts_iter)?;

        if vesting_account.data.borrow().len() < VestingScheduleInfo::LEN {
            return Err(ProgramError::InvalidAccountData)
        }
        let vesting_account_key = Pubkey::create_program_address(&[&derived_vesting_address], program_id)?;
        let state = VestingScheduleInfo::unpack(
            &vesting_account.data.borrow()[..VestingScheduleInfo::LEN],
        )?;

        if vesting_account_key != *vesting_account.key {
            msg!("Error: please check the vesting account key.");
            return Err(ProgramError::InvalidArgument);
        }

        if state.destination_address != *destination_token_account.key {
            msg!("Error: the contract destination account does not matched provided account.");
            return Err(ProgramError::InvalidArgument);
        }

        if !destination_token_account_owner.is_signer {
            msg!("Error: destination token account owner should be a signer.");
            return Err(ProgramError::InvalidArgument);
        }

        let destination_token_account = Account::unpack(&destination_token_account.data.borrow())?;

        if destination_token_account.owner != *destination_token_account_owner.key {
            msg!("Error: the current destination token account is not owned by the proper provided owner");
            return Err(ProgramError::InvalidArgument);
        }

        let mut new_state = state;
        new_state.destination_address = *new_destination_token_account.key;
        new_state
            .pack_into_slice(&mut vesting_account.data.borrow_mut()[..VestingScheduleInfo::LEN]);

        Ok(())
    }
    ///Process instruction
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Processing");
        let instruction = VestingInstruction::unpack(instruction_data)?;
        msg!("Unpacked Instruction");
        match instruction {
            VestingInstruction::Initialize {
                derived_vesting_address,
                number_of_schedules,
            } => {
                msg!("Instruction: Initialize ");
                Self::process_init(program_id, accounts, derived_vesting_address, number_of_schedules)
            }
            VestingInstruction::Unlock { derived_vesting_address } => {
                msg!("Instruction: Unlock");
                Self::process_unlock(program_id, accounts, derived_vesting_address)
            }
            VestingInstruction::ChangeDestination { derived_vesting_address } => {
                msg!("Instruction: Change Destination");
                Self::process_change_destination(program_id, accounts, derived_vesting_address)
            }
            VestingInstruction::Create {
                derived_vesting_address,
                mint_address,
                destination_token_address,
                schedules,
            } => {
                msg!("Instruction: Create Schedule");
                Self::process_create(
                    program_id,
                    accounts,
                    derived_vesting_address,
                    &mint_address,
                    &destination_token_address,
                    schedules,
                )
            }
        }
    }
}

impl PrintProgramError for VestingError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            VestingError::BadInstruction => msg!("Error: Bad instruction!"),
        }
    }
}

