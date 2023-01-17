//! Allow the authority of a `Registrar` to revoke a subdomain

use crate::{
    cpi::Cpi,
    error::SubRegisterError,
    state::{registry::Registrar, subrecord::SubRecord, Tag},
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
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    #[cons(writable)]
    /// The registrar account
    pub registrar: &'a T,

    #[cons(writable)]
    /// The subdomain account to create
    pub sub_domain_account: &'a T,

    #[cons(writable)]
    /// The subrecord account
    pub sub_record: &'a T,

    /// The current sub domain owner
    pub sub_owner: &'a T,

    /// The parent domain
    pub parent_domain: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub authority: &'a T,

    /// Name class
    pub name_class: &'a T,

    /// The name service program ID
    pub spl_name_service: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            registrar: next_account_info(accounts_iter)?,
            sub_domain_account: next_account_info(accounts_iter)?,
            sub_record: next_account_info(accounts_iter)?,
            sub_owner: next_account_info(accounts_iter)?,
            parent_domain: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
            name_class: next_account_info(accounts_iter)?,
            spl_name_service: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.name_class, &Pubkey::default())?;
        check_account_key(accounts.spl_name_service, &spl_name_service::ID)?;

        // Check owners
        check_account_owner(accounts.registrar, program_id)?;
        check_account_owner(accounts.sub_domain_account, &spl_name_service::ID)?;
        check_account_owner(accounts.sub_record, program_id)?;
        check_account_owner(accounts.parent_domain, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.authority)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut sub_record = SubRecord::from_account_info(accounts.sub_record, Tag::SubRecord)?;
    let mut registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;

    let (subrecord_key, _) = SubRecord::find_key(accounts.sub_domain_account.key, program_id);

    check_account_key(accounts.authority, &registrar.authority)?;
    check_account_key(accounts.sub_record, &subrecord_key)?;
    check_account_key(accounts.parent_domain, &registrar.domain_account)?;

    if !registrar.allow_revoke {
        return Err(SubRegisterError::CannotRevoke.into());
    }

    // Transfer to registrar
    Cpi::transfer_subdomain(
        &registrar,
        accounts.registrar,
        accounts.sub_domain_account,
        accounts.parent_domain,
        accounts.name_class,
        accounts.spl_name_service,
    )?;

    // Unregister domain
    let seeds: &[&[u8]] = &[
        Registrar::SEEDS,
        &registrar.domain_account.to_bytes(),
        &registrar.authority.to_bytes(),
        &[registrar.nonce],
    ];
    let ix = spl_name_service::instruction::delete(
        spl_name_service::ID,
        *accounts.sub_domain_account.key,
        *accounts.registrar.key,
        *accounts.registrar.key,
    )?;
    invoke_signed(
        &ix,
        &[
            accounts.spl_name_service.clone(),
            accounts.sub_domain_account.clone(),
            accounts.registrar.clone(),
            accounts.registrar.clone(),
        ],
        &[seeds],
    )?;

    // Close subrecord account
    sub_record.tag = Tag::ClosedSubRecord;
    sub_record.save(&mut accounts.sub_record.data.borrow_mut());

    // Zero out lamports of subrecord account
    let mut sub_record_lamports = accounts.sub_record.lamports.borrow_mut();
    let mut target_lamports = accounts.authority.lamports.borrow_mut();

    **target_lamports += **sub_record_lamports;
    **sub_record_lamports = 0;

    // Decrement nb sub created
    registrar.total_sub_created = registrar
        .total_sub_created
        .checked_sub(1)
        .ok_or(SubRegisterError::Overflow)?;

    // Serialize state
    registrar.save(&mut accounts.registrar.data.borrow_mut());

    Ok(())
}
