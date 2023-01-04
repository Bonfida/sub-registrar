//! Edit a registry

use crate::state::{registry::Registry, schedule::Schedule, Tag};

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
        program_error::ProgramError,
        pubkey::Pubkey,
    },
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
            authority: next_account_info(accounts_iter)?,
            registry: next_account_info(accounts_iter)?,
        };

        // Check keys

        // Check owners
        check_account_owner(accounts.registry, &program_id)?;

        // Check signer
        check_signer(accounts.authority)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut registry = Registry::from_account_info(accounts.authority, Tag::Registry)?;

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

    // Serialize state
    registry.save(&mut accounts.registry.data.borrow_mut());

    Ok(())
}
