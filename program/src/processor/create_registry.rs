//! Create registry
//!
use crate::{
    cpi::Cpi,
    error::SubRegisterError,
    state::{registry::Registry, schedule::Schedule, ROOT_DOMAIN_ACCOUNT},
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
        program::invoke,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_program,
    },
    spl_name_service::state::NameRecordHeader,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// An example input parameter
    pub mint: Pubkey,
    pub fee_account: Pubkey,
    pub authority: Pubkey,
    pub price_schedule: Schedule,
    pub nft_gated_collection: Option<Pubkey>,
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

    #[cons(writable, signer)]
    /// The owner of the domain name account
    pub domain_owner: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub fee_payer: &'a T,

    /// The SPL name service program ID
    pub spl_name_program_id: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            registry: next_account_info(accounts_iter)?,
            domain_name_account: next_account_info(accounts_iter)?,
            domain_owner: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            spl_name_program_id: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_program_id, &spl_name_service::ID)?;

        // Check owners
        check_account_owner(accounts.registry, &system_program::ID)?;
        check_account_owner(accounts.domain_name_account, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.domain_owner)?;
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], mut params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let (registry_key, nonce) = Registry::find_key(
        accounts.domain_name_account.key,
        &params.authority,
        program_id,
    );
    check_account_key(accounts.registry, &registry_key)?;

    // Checks
    let name_header =
        NameRecordHeader::unpack_from_slice(&accounts.domain_name_account.data.borrow())?;
    if name_header.parent_name != ROOT_DOMAIN_ACCOUNT {
        msg!("Only .sol are accepted");
        return Err(SubRegisterError::WrongNameAccount.into());
    }
    params.price_schedule.sort_by_key(|x| x.length);

    // Create Registry account
    let seeds: &[&[u8]] = &[
        Registry::SEEDS,
        &accounts.domain_name_account.key.to_bytes(),
        &params.authority.to_bytes(),
        &[nonce],
    ];
    let registry = Registry::new(
        &params.authority,
        &params.fee_account,
        &params.mint,
        accounts.domain_name_account.key,
        params.price_schedule,
        nonce,
        params.nft_gated_collection,
    );
    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.registry,
        seeds,
        registry.borsh_len(),
    )?;
    registry.save(&mut accounts.registry.data.borrow_mut());

    // Transfer domain to registry
    let ix = spl_name_service::instruction::transfer(
        spl_name_service::ID,
        registry_key,
        *accounts.domain_name_account.key,
        *accounts.domain_owner.key,
        None,
    )?;
    invoke(
        &ix,
        &[
            accounts.spl_name_program_id.clone(),
            accounts.domain_name_account.clone(),
            accounts.domain_owner.clone(),
        ],
    )?;

    Ok(())
}
