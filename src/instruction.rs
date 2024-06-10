use crate::state::CreditSettings;
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum CreditInstruction {
    Init { settings: CreditSettings },
    Add { pk: Pubkey, credit: u32 },
}
