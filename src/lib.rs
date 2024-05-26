use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
};
use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::BTreeMap;

entrypoint!(process_instruction);

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserAccount {
    pub user_count: u64,
    pub users: BTreeMap<String, u8>,
}

impl UserAccount {
    pub fn new() -> Self {
        Self {
            user_count: 0,
            users: BTreeMap::new(),
        }
    }
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut user_account = if account.data_len() == 0 {
        UserAccount::new()
    } else {
        UserAccount::try_from_slice(&account.data.borrow())?
    };

    match instruction_data[0] {
        0 => {
            // Initialize
            user_account.user_count = 0;
            user_account.users = BTreeMap::new();
            msg!("User account initialized.");
        },
        1 => {
            // Add user
            let name_length = instruction_data[1] as usize;
            let name = String::from_utf8(instruction_data[2..2 + name_length].to_vec()).unwrap();
            let age = instruction_data[2 + name_length];
            user_account.users.insert(name.clone(), age);
            user_account.user_count += 1;
            msg!("User {} with age {} added.", name, age);
        },
        2 => {
            // Get user age
            let name_length = instruction_data[1] as usize;
            let name = String::from_utf8(instruction_data[2..2 + name_length].to_vec()).unwrap();
            if let Some(age) = user_account.users.get(&name) {
                msg!("User: {}, Age: {}", name, age);
            } else {
                msg!("User not found.");
            }
        },
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    user_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    Ok(())
}
