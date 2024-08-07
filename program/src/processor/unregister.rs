//! Unregister a subdomain

use crate::{
    error::SubRegisterError,
    state::{mint_record::MintRecord, registry::Registrar, subdomain_record::SubDomainRecord, Tag},
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

    #[cons(writable)]
    /// The subrecord account
    pub sub_record: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub domain_owner: &'a T,

    #[cons(writable)]
    pub mint_record: Option<&'a T>,
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
            sub_record: next_account_info(accounts_iter)?,
            domain_owner: next_account_info(accounts_iter)?,
            mint_record: next_account_info(accounts_iter).ok(),
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_service, &spl_name_service::ID)?;

        // Check owners
        check_account_owner(accounts.registrar, program_id)?;
        check_account_owner(accounts.sub_domain_account, &spl_name_service::ID)?;
        check_account_owner(accounts.sub_record, program_id)?;

        // Check signer
        check_signer(accounts.domain_owner)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;
    let mut sub_record = SubDomainRecord::from_account_info(accounts.sub_record, Tag::SubRecord)?;

    let (subrecord_key, _) = SubDomainRecord::find_key(accounts.sub_domain_account.key, program_id);
    check_account_key(accounts.sub_record, &subrecord_key)?;

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

    // Handle NFT mint record
    if let Some(mint_record) = sub_record.mint_record {
        let mint_record_account = accounts
            .mint_record
            .ok_or(SubRegisterError::MissingAccount)?;
        check_account_owner(mint_record_account, program_id)?;
        check_account_key(mint_record_account, &mint_record)?;

        let mut mint_record = MintRecord::from_account_info(mint_record_account, Tag::MintRecord)?;
        mint_record.count = mint_record
            .count
            .checked_sub(1)
            .ok_or(SubRegisterError::Overflow)?;
        mint_record.save(&mut mint_record_account.data.borrow_mut());
    }

    // Close subrecord account
    sub_record.tag = Tag::ClosedSubRecord;
    sub_record.save(&mut accounts.sub_record.data.borrow_mut());

    // Zero out lamports of subrecord account
    let mut sub_record_lamports = accounts.sub_record.lamports.borrow_mut();
    let mut target_lamports = accounts.domain_owner.lamports.borrow_mut();

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
