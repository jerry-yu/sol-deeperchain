use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

// /// Instructions supported by the generic Name Registry program
// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
// pub enum CreditInstruction {
//     Add {
//         credit :u32,
//     },
//     Update {
//         credit :u32,
//     },
// }
