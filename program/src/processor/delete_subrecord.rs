//! Delete a subrecord account account
use crate::state::{subrecord::SubRecord, Tag};

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
    /// The sub domain account
    pub sub_domain: &'a T,

    #[cons(writable)]
    /// The sub record account
    pub sub_record: &'a T,

    #[cons(writable)]
    /// The lamports target
    pub lamports_target: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            sub_domain: next_account_info(accounts_iter)?,
            sub_record: next_account_info(accounts_iter)?,
            lamports_target: next_account_info(accounts_iter)?,
        };

        // Check keyss

        // Check owners
        check_account_owner(accounts.sub_domain, &system_program::ID)?;
        check_account_owner(accounts.sub_record, program_id)?;

        // Check signer

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut sub_record = SubRecord::from_account_info(accounts.sub_record, Tag::SubRecord)?;

    // Check PDA derivation
    let (sub_record_key, _) = SubRecord::find_key(accounts.sub_domain.key, program_id);
    check_account_key(accounts.sub_record, &sub_record_key)?;

    // Close sub record account
    sub_record.tag = Tag::ClosedSubRecord;

    // Put lamports to 0
    let mut lamports = accounts.sub_record.lamports.borrow_mut();
    let mut target_lamports = accounts.lamports_target.lamports.borrow_mut();

    **target_lamports += **lamports;
    **lamports = 0;

    Ok(())
}
