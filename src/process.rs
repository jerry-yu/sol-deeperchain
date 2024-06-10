use crate::instruction::CreditInstruction;
use crate::state::CreditSettings;
use crate::state::{PrivelegeUser, UserAccount};
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

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CreditInstruction::try_from_slice(instruction_data)?;

    match instruction {
        CreditInstruction::Init { settings } => {
            // Initialize PDA with credit value
            process_init(program_id, accounts, settings)?;
        }
        CreditInstruction::Add {
            pk,
            credit,
        } => {
            process_credit(program_id, accounts, pk, credit)?;
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

pub(crate) fn process_init(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    settings: CreditSettings,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    //let user_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let (pda, bump_seed) = Pubkey::find_program_address(&[b"setting"], program_id);

    if pda != *pda_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    if !pda_account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let rent = Rent::get()?;
    let setting_data = to_vec(&settings)?;
    let ulen = setting_data.len();
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
        &[&[b"setting", &[bump_seed]]],
    )?;

    let mut data = pda_account.data.borrow_mut();
    data.copy_from_slice(&setting_data);

    //user_data.serialize(&mut &mut pda_account.data.borrow_mut()[4..])?;

    msg!("PDA account initialized with credit setting: {:?}", settings);

    Ok(())
}

pub(crate) fn process_credit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    key: Pubkey,
    credit_value: u32,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    //let user_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let (pda, bump_seed) = Pubkey::find_program_address(&[b"user", key.as_ref()], program_id);

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

        let clock = clock::Clock::get()?;
        let ts = clock.unix_timestamp;

        let user_data = UserAccount {
            credit: credit_value,
            history: vec![(ts, (credit_value / 100) as u8)],
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
            &[&[b"user", key.as_ref(), &[bump_seed]]],
        )?;
        let buf = to_vec(&user_data)?;
        let real_len = buf.len();

        let mut data = pda_account.data.borrow_mut();
        data[0..4].copy_from_slice(&(real_len as u32).to_be_bytes());
        data[4..4 + real_len].copy_from_slice(&buf);
    } else {
        let credit_bytes: [u8; 4] = pda_account.data.borrow()[0..4]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let len = u32::from_be_bytes(credit_bytes);

        let mut user_data =
            UserAccount::try_from_slice(&pda_account.data.borrow()[4..4 + len as usize])?;

        user_data.credit = credit_value;

        let clock = clock::Clock::get()?;
        let ts = clock.unix_timestamp;
        user_data.history.push((ts, (credit_value / 100) as u8));

        msg!("{:?}", user_data);

        let buf = to_vec(&user_data)?;
        let real_len = buf.len();

        let mut data = pda_account.data.borrow_mut();
        data[0..4].copy_from_slice(&(real_len as u32).to_be_bytes());
        data[4..4 + real_len].copy_from_slice(&buf);
    }

    msg!("{:?} {:?} {:?}", pda, key, credit_value);

    //user_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    msg!("PDA account credit updated to: {}", credit_value);
    Ok(())
}
