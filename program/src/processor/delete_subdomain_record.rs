//! Delete a subrecord account account
use solana_program::{clock::Clock, sysvar::Sysvar};

use crate::{
    error::SubRegisterError,
    state::{mint_record::MintRecord, registry::Registrar, subdomain_record::SubDomainRecord, Tag},
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    #[cons(writable)]
    pub registrar: &'a T,

    #[cons(writable)]
    /// The sub domain account
    pub sub_domain: &'a T,

    #[cons(writable)]
    /// The sub record account
    pub sub_record: &'a T,

    #[cons(writable)]
    /// The lamports target
    pub lamports_target: &'a T,

    #[cons(writable)]
    /// The mint record account
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
            sub_domain: next_account_info(accounts_iter)?,
            sub_record: next_account_info(accounts_iter)?,
            lamports_target: next_account_info(accounts_iter)?,
            mint_record: next_account_info(accounts_iter).ok(),
        };

        // Check keys

        // Check owners
        check_account_owner(accounts.registrar, program_id)?;
        check_account_owner(accounts.sub_domain, &system_program::ID)?;
        check_account_owner(accounts.sub_record, program_id)?;

        // Check signer

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut sub_record = SubDomainRecord::from_account_info_opt(accounts.sub_record, None)?;
    match sub_record.tag {
        Tag::SubRecord => (),
        Tag::RevokedSubRecord => {
            if Clock::get()?.unix_timestamp < sub_record.expiry_timestamp {
                return Err(SubRegisterError::RevokedSubdomainNotExpired.into());
            }
        }
        _ => return Err(SubRegisterError::DataTypeMismatch.into()),
    };
    let mut registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;

    // Check PDA derivation
    let (sub_record_key, _) = SubDomainRecord::find_key(accounts.sub_domain.key, program_id);
    check_account_key(accounts.sub_record, &sub_record_key)?;

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

    // Close sub record account
    sub_record.tag = Tag::ClosedSubRecord;

    // Put lamports to 0
    let mut lamports = accounts.sub_record.lamports.borrow_mut();
    let mut target_lamports = accounts.lamports_target.lamports.borrow_mut();

    **target_lamports += **lamports;
    **lamports = 0;

    // Edit Registrar
    registrar.total_sub_created = registrar
        .total_sub_created
        .checked_sub(1)
        .ok_or(SubRegisterError::Overflow)?;
    registrar.save(&mut accounts.registrar.data.borrow_mut());

    Ok(())
}
