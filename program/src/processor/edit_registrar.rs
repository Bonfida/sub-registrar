//! Edit a registrar

use crate::{
    state::{registry::Registrar, schedule::Price, Tag},
    utils::is_price_schedule_sorted,
};

use {
    bonfida_utils::checks::check_account_owner,
    bonfida_utils::{
        checks::{check_account_key, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction, system_program,
        sysvar::Sysvar,
    },
    std::cmp::Ordering,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// The new registry
    pub new_authority: Option<Pubkey>,
    pub new_mint: Option<Pubkey>,
    pub new_fee_account: Option<Pubkey>,
    pub new_price_schedule: Option<Vec<u8>>,
    pub new_max_nft_mint: Option<u8>,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub authority: &'a T,

    #[cons(writable)]
    /// The registry to edit
    pub registrar: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
            registrar: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check owners
        check_account_owner(accounts.registrar, program_id)?;

        // Check signer
        check_signer(accounts.authority)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;

    check_account_key(accounts.authority, &registrar.authority)?;

    if let Some(new_authority) = params.new_authority {
        registrar.authority = new_authority;
    }

    if let Some(new_mint) = params.new_mint {
        registrar.mint = new_mint;
    }

    if let Some(new_fee_account) = params.new_fee_account {
        registrar.fee_account = new_fee_account;
    }

    if let Some(new_price_schedule_ser) = params.new_price_schedule {
        let new_price_schedule: Vec<Price> =
            BorshDeserialize::deserialize(&mut new_price_schedule_ser.as_slice())?;
        let sorted = is_price_schedule_sorted(&new_price_schedule);
        if !sorted {
            msg!("The schedule price array should be sorted!");
            return Err(ProgramError::InvalidArgument);
        }
        registrar.price_schedule = new_price_schedule;
    }

    if let Some(new_max_nft_mint) = params.new_max_nft_mint {
        registrar.max_nft_mint = new_max_nft_mint;
    }

    // Handle realloc
    match registrar.borsh_len().cmp(&accounts.registrar.data_len()) {
        Ordering::Greater => {
            msg!("[+] Realloc registry account (increasing size)");
            let new_lamports = Rent::get()?.minimum_balance(registrar.borsh_len());
            let diff_lamports = new_lamports.checked_sub(accounts.registrar.lamports());

            accounts.registrar.realloc(registrar.borsh_len(), false)?;

            if let Some(diff_lamports) = diff_lamports {
                let ix = system_instruction::transfer(
                    accounts.authority.key,
                    accounts.registrar.key,
                    diff_lamports,
                );
                invoke(
                    &ix,
                    &[
                        accounts.system_program.clone(),
                        accounts.authority.clone(),
                        accounts.registrar.clone(),
                    ],
                )?;
            }
        }
        Ordering::Less => {
            msg!("[+] Realloc registry account (decreasing size)");
            let new_lamports = Rent::get()?.minimum_balance(registrar.borsh_len());
            let diff_lamports = accounts.registrar.lamports().checked_sub(new_lamports);

            accounts.registrar.realloc(registrar.borsh_len(), true)?;

            if let Some(diff_lamports) = diff_lamports {
                let mut registrar_lamports = accounts.registrar.lamports.borrow_mut();
                let mut authority_lamports = accounts.authority.lamports.borrow_mut();

                **authority_lamports += diff_lamports;
                **registrar_lamports -= diff_lamports;
            }
        }
        Ordering::Equal => (),
    }

    // Serialize state
    registrar.save(&mut accounts.registrar.data.borrow_mut());

    Ok(())
}
