use crate::state::{CreditSettings, TokenAccount};
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
    Init {
        settings: CreditSettings,
        token: TokenAccount,
    },
    Add {
        pk: Pubkey,
        campaign: u16,
        credit: i32,
        //maybe removed,now for test purpose
        reward_since: u32,
    },
    SetTokenAddress {
        address: Pubkey,
    },
    Claim,
}
