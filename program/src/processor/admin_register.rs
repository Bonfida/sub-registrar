//! Allow the authority of a `Registrar` to register a subdomain without token transfer
use crate::{
    cpi::Cpi,
    error::SubRegisterError,
    state::{registry::Registrar, subrecord::SubRecord, Tag, ROOT_DOMAIN_ACCOUNT},
};
use sns_registrar::processor::create_reverse;

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    sns_registrar::instruction_auto::create_reverse,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        hash::hashv,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_program, sysvar,
        sysvar::Sysvar,
    },
    spl_name_service::state::{get_seeds_and_key, NameRecordHeader, HASH_PREFIX},
};

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

    /// The sns registrar program account
    pub sns_registrar_program: &'a T,

    /// The .sol root domain
    pub root_domain: &'a T,

    /// The reverse lookup class accoutn
    pub reverse_lookup_class: &'a T,

    /// The registrar account
    #[cons(writable)]
    pub registrar: &'a T,

    #[cons(writable)]
    /// The parent domain account
    pub parent_domain_account: &'a T,

    #[cons(writable)]
    /// The subdomain account to create
    pub sub_domain_account: &'a T,

    #[cons(writable)]
    /// The subdomain reverse account
    pub sub_reverse_account: &'a T,

    #[cons(writable)]
    /// The subrecord account
    pub sub_record: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub authority: &'a T,
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
            sns_registrar_program: next_account_info(accounts_iter)?,
            root_domain: next_account_info(accounts_iter)?,
            reverse_lookup_class: next_account_info(accounts_iter)?,
            registrar: next_account_info(accounts_iter)?,
            parent_domain_account: next_account_info(accounts_iter)?,
            sub_domain_account: next_account_info(accounts_iter)?,
            sub_reverse_account: next_account_info(accounts_iter)?,
            sub_record: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.spl_name_service, &spl_name_service::ID)?;
        check_account_key(accounts.rent_sysvar, &sysvar::rent::id())?;
        check_account_key(accounts.sns_registrar_program, &sns_registrar::ID)?;
        check_account_key(accounts.root_domain, &ROOT_DOMAIN_ACCOUNT)?;
        check_account_key(
            accounts.reverse_lookup_class,
            &sns_registrar::central_state::KEY,
        )?;

        // Check owners
        check_account_owner(accounts.registrar, program_id)?;
        check_account_owner(accounts.parent_domain_account, &spl_name_service::ID)?;
        check_account_owner(accounts.sub_domain_account, &system_program::ID)?;
        check_account_owner(accounts.sub_reverse_account, &system_program::ID).or_else(|_| {
            check_account_owner(accounts.sub_reverse_account, &spl_name_service::ID)
        })?;
        check_account_owner(accounts.sub_record, &system_program::ID)?;

        // Check signer
        check_signer(accounts.authority)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let (subrecord_key, subrecord_nonce) =
        SubRecord::find_key(accounts.sub_domain_account.key, program_id);
    let mut registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;

    check_account_key(accounts.authority, &registrar.authority)?;
    check_account_key(accounts.parent_domain_account, &registrar.domain_account)?;
    check_account_key(accounts.sub_record, &subrecord_key)?;

    if !params.domain.starts_with('\0') {
        return Err(SubRegisterError::InvalidSubdomain.into());
    }

    if params.domain.trim().to_lowercase() != params.domain {
        return Err(SubRegisterError::InvalidSubdomain.into());
    }

    if params.domain.contains('.') {
        return Err(SubRegisterError::InvalidSubdomain.into());
    }

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
        Some(&registrar.domain_account),
    );
    check_account_key(accounts.sub_domain_account, &name_account_key)?;

    // Create sub
    let space: u32 = 1_000;
    let lamports = Rent::get()?.minimum_balance(space as usize + NameRecordHeader::LEN);
    let ix = spl_name_service::instruction::create(
        spl_name_service::ID,
        spl_name_service::instruction::NameRegistryInstruction::Create {
            hashed_name,
            lamports,
            space,
        },
        *accounts.sub_domain_account.key,
        *accounts.authority.key,
        *accounts.authority.key,
        None,
        Some(registrar.domain_account),
        Some(*accounts.registrar.key),
    )?;

    let seeds: &[&[u8]] = &[
        Registrar::SEEDS,
        &registrar.domain_account.to_bytes(),
        &registrar.authority.to_bytes(),
        &[registrar.nonce],
    ];
    invoke_signed(
        &ix,
        &[
            accounts.spl_name_service.clone(),
            accounts.system_program.clone(),
            accounts.authority.clone(),
            accounts.sub_domain_account.clone(),
            accounts.authority.clone(),
            accounts.parent_domain_account.clone(),
            accounts.registrar.clone(),
        ],
        &[seeds],
    )?;

    // Sub reverse should be passed in the accounts and check if does not already exist
    if accounts.sub_reverse_account.data_is_empty() {
        let ix = create_reverse(
            sns_registrar::ID,
            create_reverse::Accounts {
                naming_service_program: &spl_name_service::ID,
                root_domain: &ROOT_DOMAIN_ACCOUNT,
                reverse_lookup: accounts.sub_reverse_account.key,
                system_program: &system_program::ID,
                central_state: &sns_registrar::central_state::KEY,
                fee_payer: accounts.authority.key,
                rent_sysvar: accounts.rent_sysvar.key,
                parent_name: Some(accounts.parent_domain_account.key),
                parent_name_owner: Some(accounts.registrar.key),
            },
            create_reverse::Params {
                name: params.domain,
            },
        );
        invoke_signed(
            &ix,
            &[
                accounts.sns_registrar_program.clone(),
                accounts.spl_name_service.clone(),
                accounts.rent_sysvar.clone(),
                accounts.spl_name_service.clone(),
                accounts.root_domain.clone(),
                accounts.sub_reverse_account.clone(),
                accounts.system_program.clone(),
                accounts.reverse_lookup_class.clone(),
                accounts.authority.clone(),
                accounts.parent_domain_account.clone(),
                accounts.registrar.clone(),
            ],
            &[seeds],
        )?;
    }

    // Create subrecord account
    let sub_record = SubRecord::new(*accounts.registrar.key);
    let seeds: &[&[u8]] = &[
        SubRecord::SEEDS,
        &accounts.sub_domain_account.key.to_bytes(),
        &[subrecord_nonce],
    ];
    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.authority,
        accounts.sub_record,
        seeds,
        sub_record.borsh_len(),
    )?;
    sub_record.save(&mut accounts.sub_record.data.borrow_mut());

    // Increment nb sub created
    registrar.total_sub_created = registrar
        .total_sub_created
        .checked_add(1)
        .ok_or(SubRegisterError::Overflow)?;

    // Serialize state
    registrar.save(&mut accounts.registrar.data.borrow_mut());

    Ok(())
}
