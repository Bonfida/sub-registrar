//! Unregister a subdomain

use crate::{
    error::SubRegisterError,
    state::{registry::Registrar, Tag},
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
        program::invoke,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_program,
    },
    spl_name_service::{instruction::delete, state::NameRecordHeader},
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    /// The SPL name service program account
    pub spl_name_service: &'a T,

    #[cons(writable)]
    /// The registrar account
    pub registrar: &'a T,

    #[cons(writable)]
    /// The subdomain account to unregister
    pub sub_domain_account: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub domain_owner: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            spl_name_service: next_account_info(accounts_iter)?,
            registrar: next_account_info(accounts_iter)?,
            sub_domain_account: next_account_info(accounts_iter)?,
            domain_owner: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_service, &spl_name_service::ID)?;

        // Check owners
        check_account_owner(accounts.registrar, program_id)?;
        check_account_owner(accounts.sub_domain_account, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.domain_owner)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;

    // Check
    let record = NameRecordHeader::unpack_from_slice(&accounts.sub_domain_account.data.borrow())?;
    if record.parent_name != registrar.domain_account {
        return Err(SubRegisterError::InvalidSubdomain.into());
    }

    // Delete sub but keep the reverse
    let ix = delete(
        spl_name_service::ID,
        *accounts.sub_domain_account.key,
        *accounts.domain_owner.key,
        *accounts.domain_owner.key,
    )?;
    invoke(
        &ix,
        &[
            accounts.spl_name_service.clone(),
            accounts.sub_domain_account.clone(),
            accounts.domain_owner.clone(),
            accounts.domain_owner.clone(),
        ],
    )?;

    // Increment nb sub created
    registrar.total_sub_created = registrar
        .total_sub_created
        .checked_sub(1)
        .ok_or(SubRegisterError::Overflow)?;

    // Serialize state
    registrar.save(&mut accounts.registrar.data.borrow_mut());

    Ok(())
}
