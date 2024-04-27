//! In the case of ...

use mpl_token_metadata::accounts::Metadata;

use crate::{
    error::SubRegisterError,
    revoke_unchecked,
    state::{mint_record::MintRecord, registry::Registrar, subdomain_record::SubDomainRecord, Tag},
    utils::{check_metadata, check_nft_holding_and_get_mint},
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
    pub sub_owner: &'a T,

    /// The parent domain
    pub parent_domain: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub nft_owner: &'a T,

    /// The NFT account
    pub nft_account: &'a T,

    pub nft_metadata: &'a T,

    #[cons(writable)]
    pub nft_mint_record: &'a T,

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
            nft_owner: next_account_info(accounts_iter)?,
            nft_account: next_account_info(accounts_iter)?,
            nft_metadata: next_account_info(accounts_iter)?,
            nft_mint_record: next_account_info(accounts_iter)?,
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
        check_account_owner(accounts.nft_account, &spl_token::ID)?;
        check_account_owner(accounts.nft_metadata, &mpl_token_metadata::ID)?;
        check_account_owner(accounts.nft_mint_record, program_id)?;

        // Check signer
        check_signer(accounts.nft_owner)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let sub_record = SubDomainRecord::from_account_info(accounts.sub_record, Tag::SubRecord)?;
    let registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;
    let mint_record = MintRecord::from_account_info(accounts.nft_mint_record, Tag::MintRecord)?;
    let collection = registrar
        .nft_gated_collection
        .ok_or(SubRegisterError::MustHaveCollection)?;

    let mint = check_nft_holding_and_get_mint(accounts.nft_account, accounts.nft_owner.key)?;
    check_metadata(accounts.nft_metadata, &collection)?;

    let (pda, _) = Metadata::find_pda(&mint);
    check_account_key(accounts.nft_metadata, &pda)?;
    let (subrecord_key, _) = SubDomainRecord::find_key(accounts.sub_domain_account.key, program_id);

    check_account_key(accounts.sub_record, &subrecord_key)?;
    check_account_key(accounts.parent_domain, &registrar.domain_account)?;

    if let Some(sub_mint_rec) = sub_record.mint_record {
        check_account_key(accounts.nft_mint_record, &sub_mint_rec)?;
        if mint != mint_record.mint {
            return Err(SubRegisterError::WrongMint.into());
        }
    } else {
        return Err(SubRegisterError::WrongMintRecord.into());
    }

    revoke_unchecked::revoke_unchecked(
        registrar,
        sub_record,
        Some(mint_record),
        accounts.registrar,
        accounts.sub_domain_account,
        accounts.parent_domain,
        accounts.name_class,
        accounts.spl_name_service,
        accounts.sub_record,
        accounts.nft_owner,
        Some(accounts.nft_mint_record),
    )?;

    Ok(())
}
