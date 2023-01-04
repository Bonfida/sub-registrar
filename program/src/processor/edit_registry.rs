//! Edit a registry

use solana_program::{
    program::{invoke, invoke_signed},
    rent::Rent,
    system_instruction, system_program,
};

use crate::{
    error::SubRegisterError,
    state::{registry::Registry, schedule::Schedule, Tag},
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
        program_error::ProgramError,
        pubkey::Pubkey,
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
    pub new_price_schedule: Option<Schedule>,
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
    pub registry: &'a T,
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
            registry: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check owners
        check_account_owner(accounts.registry, program_id)?;

        // Check signer
        check_signer(accounts.authority)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut registry = Registry::from_account_info(accounts.registry, Tag::Registry)?;

    check_account_key(accounts.authority, &registry.authority)?;

    // FIXME: account size issues with different length in price schedule

    if let Some(new_authority) = params.new_authority {
        registry.authority = new_authority;
    }

    if let Some(new_mint) = params.new_mint {
        registry.mint = new_mint;
    }

    if let Some(new_fee_account) = params.new_fee_account {
        registry.fee_account = new_fee_account;
    }

    if let Some(new_price_schedule) = params.new_price_schedule {
        registry.price_schedule = new_price_schedule;
    }

    // Handle realloc
    match registry.borsh_len().cmp(&accounts.registry.data_len()) {
        Ordering::Greater => {
            msg!("[+] Realloc registry account (increasing size)");
            let new_lamports = Rent::get()?.minimum_balance(registry.borsh_len());
            let diff_lamports = new_lamports
                .checked_sub(accounts.registry.lamports())
                .ok_or(SubRegisterError::Overflow)?;

            accounts.registry.realloc(registry.borsh_len(), false)?;

            let ix = system_instruction::transfer(
                accounts.authority.key,
                accounts.registry.key,
                diff_lamports,
            );
            invoke(
                &ix,
                &[
                    accounts.system_program.clone(),
                    accounts.authority.clone(),
                    accounts.registry.clone(),
                ],
            )?;
        }
        Ordering::Less => {
            msg!("[+] Realloc registry account (decreasing size)");
            let new_lamports = Rent::get()?.minimum_balance(registry.borsh_len());
            let diff_lamports = accounts
                .registry
                .lamports()
                .checked_sub(new_lamports)
                .ok_or(SubRegisterError::Overflow)?;

            accounts.registry.realloc(registry.borsh_len(), true)?;

            let mut registry_lamports = accounts.registry.lamports.borrow_mut();
            let mut authority_lamports = accounts.authority.lamports.borrow_mut();

            **authority_lamports += diff_lamports;
            **registry_lamports -= diff_lamports;
        }
        Ordering::Equal => (),
    }

    // Serialize state
    registry.save(&mut accounts.registry.data.borrow_mut());

    Ok(())
}
