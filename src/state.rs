use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct UserAccount {
    pub credit: u32,
    pub history: Vec<(i64, u8)>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Clone, PartialEq)]
pub struct CreditSetting {
    pub campain_id: u16,
    pub level: u8,
    pub daily_reward: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Clone, PartialEq)]
pub struct CreditSettings {
    pub settings: Vec<CreditSetting>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct PrivelegeUser {
    pub users: Vec<Pubkey>,
}
