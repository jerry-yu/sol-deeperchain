use crate::{process, state::{PrivelegeUser, UserAccount}};
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data[0] {
        0 => {
            // Initialize PDA with credit value
            process::process_init(program_id,accounts,instruction_data)?;
        }
        1 => {
            process::process_credit(program_id,accounts,instruction_data)?;
        }
        // 2 => {
        //     let key = Pubkey::try_from(&instruction_data[1..33]).unwrap();
        //     // Query credit value
        //     let (pda, _bump_seed) = Pubkey::find_program_address(
        //         &[b"user", key.as_ref()],
        //         program_id,
        //     );

        //     if pda != *pda_account.key {
        //         return Err(ProgramError::InvalidArgument);
        //     }

        //     let user_data = UserAccount::try_from_slice(&pda_account.data.borrow())?;
        //     msg!("Credit value: {}", user_data.credit);
        // }
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}
