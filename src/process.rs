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
    sysvar::{clock, rent::Rent, Sysvar},
};
use spl_associated_token_account::{
    get_associated_token_address, instruction as associated_token_account_instruction,
};
use spl_token::instruction as token_instruction;

use crate::{instruction::CreditInstruction, MINT_AUTHORITY_SEED};
use crate::state::{CreditSettings, UserCredit};
use crate::state::{PrivelegeUser, TokenAccount, UserAccount};
use crate::{CREDIT_SETTING_SEED, TOKEN_SEED, USER_CREDIT_SEED};

const SECS_PER_DAY: i64 = 60 * 60 * 24;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CreditInstruction::try_from_slice(instruction_data)?;

    match instruction {
        CreditInstruction::Init { settings, token} => {
            // Initialize PDA with credit value
            process_init(program_id, accounts, settings, token,)?;
        }
        CreditInstruction::Add {
            pk,
            credit,
            campaign,
            reward_since,
        } => {
            process_credit(program_id, accounts, pk, credit, campaign, reward_since)?;
        }
        CreditInstruction::SetTokenAddress { address } => {
            process_token_address(program_id, accounts, address)?;
        }
        CreditInstruction::Claim => {
            process_claim(program_id, accounts)?;
        }

        _ => return Err(ProgramError::InvalidInstructionData),
    }
    Ok(())
}

pub(crate) fn process_init(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    settings: CreditSettings,
    token: TokenAccount,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    //let user_account = next_account_info(accounts_iter)?;
    let setting_account = next_account_info(accounts_iter)?;
    let dpr_account = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;

    let system_program = next_account_info(accounts_iter)?;

    let (setting_key, setting_seed) =
        Pubkey::find_program_address(&[CREDIT_SETTING_SEED], program_id);
    if setting_key != *setting_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let (dpr_key, dpr_seed) = Pubkey::find_program_address(&[TOKEN_SEED], program_id);
    if dpr_key != *dpr_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let (mint_key, mint_seed) = Pubkey::find_program_address(&[MINT_AUTHORITY_SEED], program_id);
    if mint_key != *mint_authority.key {
        return Err(ProgramError::InvalidArgument);
    }

    if !setting_account.data_is_empty() || !dpr_account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let rent = Rent::get()?;

    // init credit setting
    let setting_data = to_vec(&settings)?;
    let ulen = setting_data.len();
    let required_lamports = rent.minimum_balance(ulen);

    let create_account_ix = system_instruction::create_account(
        payer_account.key,
        setting_account.key,
        required_lamports,
        ulen as u64,
        program_id,
    );

    invoke_signed(
        &create_account_ix,
        &[
            payer_account.clone(),
            setting_account.clone(),
            system_program.clone(),
        ],
        &[&[CREDIT_SETTING_SEED, &[setting_seed]]],
    )?;

    let mut data = setting_account.data.borrow_mut();
    data.copy_from_slice(&setting_data);

    // init token address
    let token_data = to_vec(&token)?;
    let ulen = token_data.len();
    let required_lamports = rent.minimum_balance(ulen);

    let create_account_ix = system_instruction::create_account(
        payer_account.key,
        dpr_account.key,
        required_lamports,
        ulen as u64,
        program_id,
    );

    invoke_signed(
        &create_account_ix,
        &[
            payer_account.clone(),
            dpr_account.clone(),
            system_program.clone(),
        ],
        &[&[TOKEN_SEED, &[dpr_seed]]],
    )?;

    let mut data = dpr_account.data.borrow_mut();
    data.copy_from_slice(&token_data);

    let required_lamports = rent.minimum_balance(0);
    let create_account_ix = system_instruction::create_account(
        payer_account.key,
        mint_authority.key,
        required_lamports,
        0,
        program_id,
    );

    invoke_signed(
        &create_account_ix,
        &[
            payer_account.clone(),
            mint_authority.clone(),
            system_program.clone(),
        ],
        &[&[MINT_AUTHORITY_SEED, &[mint_seed]]],
    )?;

    msg!(
        "PDA account initialized with credit setting: {:?} mint_authority {:?}",
        settings,mint_authority
    );

    Ok(())
}

fn add_u32_i32(u: u32, i: i32) -> u32 {
    let u = u as i64;
    let i = i as i64;
    let result = u + i;

    if result >= 0 && result <= u32::MAX as i64 {
        result as u32
    } else {
        0
    }
}

pub(crate) fn process_credit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    key: Pubkey,
    credit_value: i32,
    campaign_id: u16,
    reward_since: u32,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    //let user_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let (pda, bump_seed) =
        Pubkey::find_program_address(&[USER_CREDIT_SEED, key.as_ref()], program_id);

    msg!("pda {}", pda);
    if pda != *pda_account.key {
        return Err(ProgramError::InvalidArgument);
    }
    if pda_account.data_is_empty() {
        let credit_size = 4; // u32
        let vec_header_size = 4 + 4 + 8; // Vec header
        let element_size = 8 + 1; // (u64, u8)
        let max_elements = 100;
        let ulen = credit_size + vec_header_size + (element_size * max_elements);
        msg!("ulen {}", ulen);

        let day = (clock::Clock::get()?.unix_timestamp / SECS_PER_DAY) as u32;

        let user_data = UserAccount {
            campaign_id,
            credit: add_u32_i32(0, credit_value),
            history: vec![UserCredit {
                day,
                campaign_id,
                level: get_level(credit_value as u32),
            }],
            reward_since,
        };

        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(ulen);

        let create_account_ix = system_instruction::create_account(
            payer_account.key,
            pda_account.key,
            required_lamports,
            ulen as u64,
            program_id,
        );

        invoke_signed(
            &create_account_ix,
            &[
                payer_account.clone(),
                pda_account.clone(),
                system_program.clone(),
            ],
            &[&[USER_CREDIT_SEED, key.as_ref(), &[bump_seed]]],
        )?;

        UserAccount::pack(user_data, &mut pda_account.data.borrow_mut())?;
    } else {
        let mut user_data = UserAccount::unpack(&pda_account.data.borrow())?;
        let old_level = get_level(user_data.credit);
        user_data.credit = add_u32_i32(user_data.credit, credit_value);
        user_data.campaign_id = campaign_id;

        let new_level = get_level(user_data.credit);

        if old_level != new_level {
            let day = (clock::Clock::get()?.unix_timestamp / SECS_PER_DAY) as u32;
            user_data.history.push(UserCredit {
                day,
                campaign_id,
                level: get_level(credit_value as u32),
            });
        }
        msg!("{:?}", user_data);
        UserAccount::pack(user_data, &mut pda_account.data.borrow_mut())?;
    }
    msg!("{:?} {:?} {:?}", pda, key, credit_value);
    Ok(())
}

pub(crate) fn process_token_address(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    address: Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let dpr_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let (dpr_key, dpr_seed) = Pubkey::find_program_address(&[TOKEN_SEED], program_id);
    if dpr_key != *dpr_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let token = TokenAccount { token: address };

    token.serialize(&mut &mut dpr_account.data.borrow_mut()[..])?;
    msg!("set token address: {:?}", token);
    Ok(())
}

fn get_level(credit: u32) -> u8 {
    let lv = credit / 100;
    if lv > 8 {
        8
    } else {
        lv as u8
    }
}

fn calculate_current_earnings(
    settings: &CreditSettings,
    user_account: &UserAccount,
    current_day: u32,
) -> u64 {
    let mut total_earnings = 0;
    let mut previous_day = user_account.reward_since;
    let mut current_level = 0;
    let mut current_id = 0;

    for info in user_account.history.iter() {
        if info.day > current_day {
            break;
        }

        if info.level != 0 {
            let earnings_per_day = settings
                .settings
                .iter()
                .find(|&setting| {
                    setting.campaign_id == info.campaign_id && setting.level == info.level
                })
                .map(|setting| setting.daily_reward)
                .unwrap_or(0);
            total_earnings += earnings_per_day * (info.day.saturating_sub(previous_day)) as u64;
        }
        previous_day = info.day;
        current_level = info.level;
        current_id = info.campaign_id;
    }

    if current_level != 0 {
        let earnings_per_day = settings
            .settings
            .iter()
            .find(|&setting| setting.campaign_id == current_id && setting.level == current_level)
            .map(|setting| setting.daily_reward)
            .unwrap_or(0);
        total_earnings += earnings_per_day * (current_day.saturating_sub(previous_day)) as u64;
    }
    total_earnings
}

pub(crate) fn process_claim(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let user_credit_account = next_account_info(accounts_iter)?;

    let user_associated_token_account = next_account_info(accounts_iter)?;

    let token_account = next_account_info(accounts_iter)?;
    let dpr_mint_account = next_account_info(accounts_iter)?;
    let setting_account = next_account_info(accounts_iter)?;

    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;
    let my_program = next_account_info(accounts_iter)?;

    msg!(
        "======== {:?} {:?} {:?} {:?} {:?} {:?}",
        dpr_mint_account.key,
        token_account.key,
        associated_token_program.key,
        my_program.key,
        system_program.key,
        token_program.key,
    );

    let expected_associated_token_address =
        get_associated_token_address(&payer_account.key, &dpr_mint_account.key);

    msg!(
        "&&&&&&&&&&&& {:?}  {:?}",
        expected_associated_token_address,
        user_associated_token_account.key
    );

    if *user_associated_token_account.key != expected_associated_token_address {
        return Err(ProgramError::InvalidArgument);
    }

    if user_associated_token_account.lamports() == 0 {
        msg!("Creating associated token account...");
        invoke(
            &associated_token_account_instruction::create_associated_token_account(
                payer_account.key,
                payer_account.key,
                dpr_mint_account.key,
                token_program.key,
            ),
            &[
                dpr_mint_account.clone(),
                user_associated_token_account.clone(),
                payer_account.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
            ],
        )?;
    } else {
        msg!("Associated token account exists.");
    }

    let (dpr_key, _) = Pubkey::find_program_address(&[TOKEN_SEED], program_id);
    let (user_credit_key, _) =
        Pubkey::find_program_address(&[USER_CREDIT_SEED, payer_account.key.as_ref()], program_id);
    let (setting_key, __) = Pubkey::find_program_address(&[CREDIT_SETTING_SEED], program_id);
    let (mint_key, mint_seed) = Pubkey::find_program_address(&[MINT_AUTHORITY_SEED], program_id);

    msg!(
        "---- {:?} {:?} {:?} {:?} {:?} {:?}  {:?}",
        dpr_key,
        token_account.key,
        user_credit_key,
        user_credit_account.key,
        setting_key,
        setting_account.key,mint_key
    );

    if dpr_key != *token_account.key
        || user_credit_key != *user_credit_account.key
        || setting_key != *setting_account.key
    {
        return Err(ProgramError::InvalidArgument);
    }

    let settings = CreditSettings::try_from_slice(&setting_account.data.borrow())?;
    let token = TokenAccount::try_from_slice(&token_account.data.borrow())?;
    if token.token != *dpr_mint_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let user_account = UserAccount::unpack(&mut &mut user_credit_account.data.borrow_mut()[..])?;
    msg!(
        "settings: {:?} token {:?} user_credit {:?}",
        settings,
        token,
        user_account
    );
    let day = clock::Clock::get()?.unix_timestamp / SECS_PER_DAY;
    let reward = calculate_current_earnings(&settings, &user_account, day as u32);

    msg!("getting reward {}", reward);

    invoke_signed(
        &token_instruction::mint_to(
            token_program.key,
            dpr_mint_account.key,
            user_associated_token_account.key,
            mint_authority.key,
            &[mint_authority.key],
            reward,
        )?,
        &[
            dpr_mint_account.clone(),
            mint_authority.clone(),
            user_associated_token_account.clone(),
            token_program.clone(),
        ],
        &[&[MINT_AUTHORITY_SEED, &[mint_seed]]],
    )?;
    Ok(())
}
