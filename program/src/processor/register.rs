//! Register a subdomain

use crate::{
    error::SubRegisterError,
    state::{registry::Registry, Tag},
    utils,
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    name_auctioning::instructions::create_reverse,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        hash::hashv,
        msg,
        program::invoke,
        program::invoke_signed,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey,
        pubkey::Pubkey,
        rent::Rent,
        system_program, sysvar,
        sysvar::Sysvar,
    },
    spl_name_service::state::{get_seeds_and_key, NameRecordHeader, HASH_PREFIX},
};

pub const NAME_AUCTIONING: Pubkey = pubkey!("jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR");

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// The subdomain to register
    pub domain: String,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The SPL name service program account
    pub spl_name_service: &'a T,

    /// The rent sysvar account
    pub rent_sysvar: &'a T,

    /// The name auctioning program account
    pub name_auctioning_program: &'a T,

    /// The .sol root domain
    pub root_domain: &'a T,

    /// The reverse lookup class accoutn
    pub reverse_lookup_class: &'a T,

    /// The fee account of the registry
    #[cons(writable)]
    pub fee_account: &'a T,

    #[cons(writable)]
    pub fee_source: &'a T,

    #[cons(writable)]
    pub registry: &'a T,

    #[cons(writable)]
    pub parent_domain_account: &'a T,

    #[cons(writable)]
    pub sub_domain_account: &'a T,

    #[cons(writable)]
    pub sub_reverse_account: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub fee_payer: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            spl_name_service: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            name_auctioning_program: next_account_info(accounts_iter)?,
            root_domain: next_account_info(accounts_iter)?,
            reverse_lookup_class: next_account_info(accounts_iter)?,
            fee_account: next_account_info(accounts_iter)?,
            fee_source: next_account_info(accounts_iter)?,
            registry: next_account_info(accounts_iter)?,
            parent_domain_account: next_account_info(accounts_iter)?,
            sub_domain_account: next_account_info(accounts_iter)?,
            sub_reverse_account: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_service, &spl_name_service::ID)?;
        check_account_key(accounts.rent_sysvar, &sysvar::rent::id())?;
        check_account_key(accounts.name_auctioning_program, &NAME_AUCTIONING)?;

        // Check owners
        check_account_owner(accounts.fee_source, &spl_token::ID)?;
        check_account_owner(accounts.registry, program_id)?;
        check_account_owner(accounts.parent_domain_account, &spl_name_service::ID)?;
        check_account_owner(accounts.sub_domain_account, &system_program::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut registry = Registry::from_account_info(accounts.registry, Tag::Registry)?;

    check_account_key(accounts.fee_account, &registry.fee_account)?;
    check_account_key(accounts.parent_domain_account, &registry.domain_account)?;

    // Check sub account derivation
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &params.domain).as_bytes()])
        .as_ref()
        .to_vec();
    if hashed_name.len() != 32 {
        msg!("Invalid seed length");
        return Err(ProgramError::InvalidArgument);
    }

    let (name_account_key, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed_name.clone(),
        None,
        Some(&registry.domain_account),
    );
    check_account_key(accounts.sub_domain_account, &name_account_key)?;

    // Transfer fees
    let price = utils::get_domain_price(params.domain.clone(), &registry.price_schedule);
    let ix = spl_token::instruction::transfer(
        &spl_token::ID,
        accounts.fee_source.key,
        accounts.fee_account.key,
        accounts.fee_payer.key,
        &[],
        price,
    )?;
    invoke(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.fee_source.clone(),
            accounts.fee_account.clone(),
            accounts.fee_payer.clone(),
        ],
    )?;

    // Create sub
    let space: u32 = 1_000;
    let lamports = Rent::get()
        .unwrap()
        .minimum_balance(space as usize + NameRecordHeader::LEN);
    let ix = spl_name_service::instruction::create(
        spl_name_service::ID,
        spl_name_service::instruction::NameRegistryInstruction::Create {
            hashed_name,
            lamports,
            space,
        },
        *accounts.sub_domain_account.key,
        *accounts.fee_payer.key,
        *accounts.fee_payer.key,
        None,
        Some(registry.domain_account),
        Some(*accounts.registry.key),
    )?;

    let seeds: &[&[u8]] = &[
        Registry::SEEDS,
        &registry.domain_account.to_bytes(),
        &registry.authority.to_bytes(),
        &[registry.nonce],
    ];
    invoke_signed(
        &ix,
        &[
            accounts.spl_name_service.clone(),
            accounts.system_program.clone(),
            accounts.fee_payer.clone(),
            accounts.sub_domain_account.clone(),
            accounts.fee_payer.clone(),
            accounts.parent_domain_account.clone(),
            accounts.registry.clone(),
        ],
        &[seeds],
    )?;

    // Sub reverse should be passed in the accounts and check if does not already exist
    if accounts.sub_reverse_account.data_is_empty() {
        let ix = create_reverse(
            NAME_AUCTIONING,
            name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
            *accounts.sub_reverse_account.key,
            name_auctioning::processor::CENTRAL_STATE,
            *accounts.fee_payer.key,
            params.domain,
            Some(registry.domain_account),
            Some(*accounts.registry.key),
        );
        invoke_signed(
            &ix,
            &[
                accounts.name_auctioning_program.clone(),
                accounts.rent_sysvar.clone(),
                accounts.spl_name_service.clone(),
                accounts.root_domain.clone(),
                accounts.sub_reverse_account.clone(),
                accounts.system_program.clone(),
                accounts.reverse_lookup_class.clone(),
                accounts.fee_payer.clone(),
                accounts.parent_domain_account.clone(),
                accounts.registry.clone(),
            ],
            &[seeds],
        )?;
    }

    // Increment nb sub created
    registry.total_sub_created = registry
        .total_sub_created
        .checked_add(1)
        .ok_or(SubRegisterError::Overflow)?;

    // Serialize state
    registry.save(&mut accounts.registry.data.borrow_mut());

    Ok(())
}
