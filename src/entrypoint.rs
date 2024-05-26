use solana_program::pubkey::Pubkey;
use solana_program::instruction::{Instruction, AccountMeta};
use solana_program::system_instruction;
use solana_program_test::{processor, tokio};
use solana_sdk::{signature::Signer, transaction::Transaction, transport::TransportError};
use borsh::BorshSerialize;

pub fn initialize_account(user_account: &Pubkey, user: &Pubkey) -> Instruction {
    let data = vec![0]; // Initialize instruction
    Instruction::new_with_borsh(*user_account, &data, vec![AccountMeta::new(*user, true)])
}

pub fn add_user(user_account: &Pubkey, name: String, age: u8) -> Instruction {
    let mut data = vec![1]; // Add user instruction
    data.push(name.len() as u8);
    data.extend_from_slice(name.as_bytes());
    data.push(age);
    Instruction::new_with_borsh(*user_account, &data, vec![AccountMeta::new(*user_account, false)])
}

pub fn get_user_age(user_account: &Pubkey, name: String) -> Instruction {
    let mut data = vec![2]; // Get user age instruction
    data.push(name.len() as u8);
    data.extend_from_slice(name.as_bytes());
    Instruction::new_with_borsh(*user_account, &data, vec![AccountMeta::new_readonly(*user_account, false)])
}
