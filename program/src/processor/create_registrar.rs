//! Create registrar

use crate::{
    cpi::Cpi,
    error::SubRegisterError,
    state::{registry::Registrar, schedule::Price, ROOT_DOMAIN_ACCOUNT},
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
    pub mint: Pubkey,
    pub fee_account: Pubkey,
    pub authority: Pubkey,
    pub price_schedule: Vec<Price>,
    pub nft_gated_collection: Option<Pubkey>,
    pub max_nft_mint: u8,
    pub allow_revoke: bool,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    #[cons(writable)]
    /// The registrar account
    pub registrar: &'a T,

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
            registrar: next_account_info(accounts_iter)?,
            domain_name_account: next_account_info(accounts_iter)?,
            domain_owner: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            spl_name_program_id: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_program_id, &spl_name_service::ID)?;

        // Check owners
        check_account_owner(accounts.registrar, &system_program::ID).unwrap();
        check_account_owner(accounts.domain_name_account, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.domain_owner)?;
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let (registrar_key, nonce) = Registrar::find_key(accounts.domain_name_account.key, program_id);
    check_account_key(accounts.registrar, &registrar_key)?;

    // Checks
    let name_header =
        NameRecordHeader::unpack_from_slice(&accounts.domain_name_account.data.borrow())?;
    if name_header.parent_name != ROOT_DOMAIN_ACCOUNT {
        msg!("Only .sol are accepted");
        return Err(SubRegisterError::WrongNameAccount.into());
    }
    let sorted = params
        .price_schedule
        .iter()
        .try_fold(
            0,
            |acc, p| {
                if p.length < acc {
                    None
                } else {
                    Some(p.length)
                }
            },
        )
        .is_some();
    if !sorted {
        msg!("The schedule price array should be sorted!");
        return Err(ProgramError::InvalidArgument);
    }
    // price_schedule.sort_by_key(|x| x.length);

    // Create Registry account
    let seeds: &[&[u8]] = &[
        Registrar::SEEDS,
        &accounts.domain_name_account.key.to_bytes(),
        &params.authority.to_bytes(),
        &[nonce],
    ];
    let registry = Registrar::new(
        &params.authority,
        &params.fee_account,
        &params.mint,
        accounts.domain_name_account.key,
        params.price_schedule,
        nonce,
        params.nft_gated_collection,
        params.max_nft_mint,
        params.allow_revoke,
    );
    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.registrar,
        seeds,
        registry.borsh_len(),
    )?;
    registry.save(&mut accounts.registrar.data.borrow_mut());

    // Transfer domain to registry
    let ix = spl_name_service::instruction::transfer(
        spl_name_service::ID,
        registrar_key,
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
