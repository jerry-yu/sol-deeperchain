use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, Default)]
pub struct UserCredit {
    pub campaign_id: u16,
    pub level: u8,
    pub day: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct UserAccount {
    pub campaign_id: u16,
    pub credit: u32,
    // reward data which days clamed
    pub reward_since: u32,
    // u32: days,u16:campaign id, u8: credit level
    pub history: Vec<UserCredit>,
}

impl UserAccount {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let credit_bytes: [u8; 4] = data[0..4]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let len = u32::from_be_bytes(credit_bytes);
        UserAccount::try_from_slice(&data[4..4 + len as usize])
            .map_err(|_| ProgramError::BorshIoError("user account error".to_string()))
    }

    pub fn pack(src: Self, dst: &mut [u8]) -> Result<(), ProgramError> {
        let buf = to_vec(&src)?;
        let real_len = buf.len();
        dst[0..4].copy_from_slice(&(real_len as u32).to_be_bytes());
        dst[4..4 + real_len].copy_from_slice(&buf);
        Ok(())
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default, PartialEq)]
pub struct TokenAccount {
    pub token: Pubkey,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Clone, PartialEq)]
pub struct CreditSetting {
    pub campaign_id: u16,
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
