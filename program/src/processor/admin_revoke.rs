//! Allow the authority of a `Registrar` to revoke a subdomain

use crate::{
    error::SubRegisterError,
    revoke_unchecked,
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
    #[cons(writable)]
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
            registrar: next_account_info(accounts_iter)?,
            sub_domain_account: next_account_info(accounts_iter)?,
            sub_record: next_account_info(accounts_iter)?,
            sub_owner: next_account_info(accounts_iter)?,
            parent_domain: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
            name_class: next_account_info(accounts_iter)?,
            spl_name_service: next_account_info(accounts_iter)?,
            mint_record: next_account_info(accounts_iter).ok(),
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

    let sub_record = SubDomainRecord::from_account_info(accounts.sub_record, Tag::SubRecord)?;
    let registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;

    let (subrecord_key, _) = SubDomainRecord::find_key(accounts.sub_domain_account.key, program_id);

    check_account_key(accounts.authority, &registrar.authority)?;
    check_account_key(accounts.sub_record, &subrecord_key)?;
    check_account_key(accounts.parent_domain, &registrar.domain_account)?;

    if !registrar.allow_revoke {
        return Err(SubRegisterError::CannotRevoke.into());
    }

    let (mr, mr_acc) = match (registrar.nft_gated_collection, accounts.mint_record) {
        (None, Some(_)) | (Some(_), None) => return Err(SubRegisterError::MissingMintRecord.into()),
        (None, None) => (None, None),
        (Some(_), Some(mint_record_account)) => {
            check_account_owner(mint_record_account, program_id)?;

            let mint_record = MintRecord::from_account_info(mint_record_account, Tag::MintRecord)?;

            check_account_key(mint_record_account, &sub_record.mint_record.unwrap())?;

            (Some(mint_record), accounts.mint_record)
        }
    };

    revoke_unchecked::revoke_unchecked(
        registrar,
        sub_record,
        mr,
        accounts.registrar,
        accounts.sub_domain_account,
        accounts.parent_domain,
        accounts.name_class,
        accounts.spl_name_service,
        accounts.sub_record,
        accounts.sub_owner,
        mr_acc,
    )?;

    Ok(())
}
