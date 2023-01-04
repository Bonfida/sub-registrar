//! Close a registry account
use crate::{
    error::SubRegisterError,
    state::{registry::Registry, schedule::Schedule, Tag},
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    pub mint: Pubkey,
    pub fee_account: Pubkey,
    pub authority: Pubkey,
    pub price_schedule: Schedule,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    #[cons(writable)]
    /// The registry account
    pub registry: &'a T,

    #[cons(writable)]
    /// The domain account
    pub domain_name_account: &'a T,

    #[cons(writable)]
    /// The new owner of the domain name account
    pub new_domain_owner: &'a T,

    #[cons(writable)]
    /// The lamports target
    pub lamports_target: &'a T,

    #[cons(writable, signer)]
    /// The authority of the registry
    pub registry_authority: &'a T,

    /// The SPL name service program ID
    pub spl_name_program_id: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            registry: next_account_info(accounts_iter)?,
            domain_name_account: next_account_info(accounts_iter)?,
            new_domain_owner: next_account_info(accounts_iter)?,
            lamports_target: next_account_info(accounts_iter)?,
            registry_authority: next_account_info(accounts_iter)?,
            spl_name_program_id: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_program_id, &spl_name_service::ID)?;

        // Check owners
        check_account_owner(accounts.registry, program_id)?;
        check_account_owner(accounts.domain_name_account, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.registry_authority)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut registry = Registry::from_account_info(accounts.registry, crate::state::Tag::Registry)?;

    // Checks
    check_account_key(accounts.registry_authority, &registry.authority)?;
    check_account_key(accounts.domain_name_account, &registry.domain_account)?;

    if registry.total_sub_created != 0 {
        msg!(
            "Cannot close registry - {} subs are still registered",
            registry.total_sub_created
        );
        return Err(SubRegisterError::CannotCloseRegistry.into());
    }

    // Transfer domain to the user
    let seeds: &[&[u8]] = &[
        Registry::SEEDS,
        &accounts.domain_name_account.key.to_bytes(),
        &params.authority.to_bytes(),
        &[registry.nonce],
    ];
    let ix = spl_name_service::instruction::transfer(
        spl_name_service::ID,
        *accounts.new_domain_owner.key,
        *accounts.domain_name_account.key,
        *accounts.registry.key,
        None,
    )?;
    invoke_signed(
        &ix,
        &[
            accounts.spl_name_program_id.clone(),
            accounts.domain_name_account.clone(),
            accounts.new_domain_owner.clone(),
        ],
        &[seeds],
    )?;

    // Close registry account
    registry.tag = Tag::ClosedRegistry;
    registry.save(&mut accounts.registry.data.borrow_mut());

    // Put lamports to 0
    let mut lamports = accounts.registry.lamports.borrow_mut();
    let mut target_lamports = accounts.lamports_target.lamports.borrow_mut();

    **target_lamports += **lamports;
    **lamports = 0;

    Ok(())
}
