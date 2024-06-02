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
    sysvar::{rent::Rent, Sysvar,clock},
};

pub fn process_init(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    //let user_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let key = Pubkey::try_from(&instruction_data[1..33]).unwrap();
    let credit_value = u32::from_le_bytes(instruction_data[33..37].try_into().unwrap());
    let (pda, bump_seed) = Pubkey::find_program_address(&[b"user", key.as_ref()], program_id);

    msg!("{:?} {:?} {:?}",pda,key,credit_value);
    if pda != *pda_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let rent = Rent::get()?;
                // let ulen = std::mem::size_of::<UserAccount>();
                // let olen = to_vec(&UserAccount::default())?.len();
                // msg!("ulen---- {} --- {}", ulen, olen);

    let credit_size = 4;  // u32
    let vec_header_size = 4 + 4 + 8;  // Vec header
    let element_size = 8 + 1;  // (u64, u8)
    let max_elements = 100;  // 计划存储的最大元素数量
    let ulen = credit_size + vec_header_size + (element_size * max_elements);
    msg!("old ulen {}",ulen);
    
    let mut user_data = UserAccount {
        credit: credit_value,
        history: Vec::with_capacity(max_elements),
    };

    let clock = clock::Clock::get()?;
    let ts = clock.unix_timestamp;
    user_data.history.push((ts,(credit_value/100) as u8));

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
    
    //user_data.serialize(&mut &mut pda_account.data.borrow_mut()[4..])?;

    msg!("PDA account initialized with credit: {}", credit_value);

    Ok(())
}

pub fn process_credit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    //let user_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Update credit value (only allowed by specific user)
    let key = Pubkey::try_from(&instruction_data[1..33]).unwrap();
    let new_credit_value = u32::from_le_bytes(instruction_data[33..37].try_into().unwrap());
    
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"user", key.as_ref()], program_id);

    if pda != *pda_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    msg!("{:?} {:?} {:?}",pda,key,new_credit_value);

    let credit_bytes: [u8; 4] = pda_account.data.borrow()[0..4].try_into().map_err(|_| ProgramError::InvalidAccountData)?;

    let len = u32::from_be_bytes(credit_bytes);

    let mut user_data = UserAccount::try_from_slice(&pda_account.data.borrow()[4..4+len as usize])?;
    msg!("--------------  {:?}",user_data);
    
    user_data.credit = new_credit_value;

    let clock = clock::Clock::get()?;
    let ts = clock.unix_timestamp;
    user_data.history.push((ts,(new_credit_value/100) as u8));

    msg!("{:?}",user_data);

    let buf = to_vec(&user_data)?;
    let real_len = buf.len();

    let mut data = pda_account.data.borrow_mut();
    data[0..4].copy_from_slice(&(real_len as u32).to_be_bytes());
    data[4..4 + real_len].copy_from_slice(&buf);

    //user_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    msg!("PDA account credit updated to: {}", new_credit_value);
    Ok(())
}
