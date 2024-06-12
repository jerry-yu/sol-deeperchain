mod entrypoint;
pub mod instruction;
mod process;
pub mod state;


pub const TOKEN_SEED: &[u8] = b"dpr_token";
pub const CREDIT_SETTING_SEED: &[u8] = b"credit_setting";
pub const USER_CREDIT_SEED: &[u8] = b"user";