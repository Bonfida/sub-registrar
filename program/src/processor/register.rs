//! Register a subdomain

use mpl_token_metadata::accounts::Metadata;
use sns_registrar::processor::create_reverse;

use crate::{
    cpi::Cpi,
    error::SubRegisterError,
    state::{
        mint_record::MintRecord, registry::Registrar, subrecord::SubRecord, Tag, FEE_ACC_OWNER,
        FEE_PCT, ROOT_DOMAIN_ACCOUNT,
    },
    utils,
    utils::{check_metadata, check_nft_holding_and_get_mint},
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer, check_token_account_owner},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    sns_registrar::instruction_auto::create_reverse,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        hash::hashv,
        msg,
        program::invoke,
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

    /// The name auctioning program account
    pub sns_registrar_program: &'a T,

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
    pub registrar: &'a T,

    #[cons(writable)]
    pub parent_domain_account: &'a T,

    #[cons(writable)]
    pub sub_domain_account: &'a T,

    #[cons(writable)]
    pub sub_reverse_account: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub fee_payer: &'a T,

    #[cons(writable)]
    pub bonfida_fee_account: &'a T,

    #[cons(writable)]
    /// The subrecord account
    pub sub_record: &'a T,

    /// Optional NFT account if Registrar is NFT gated
    pub nft_account: Option<&'a T>,

    /// Optional NFT metadata account if Registrar is NFT gated
    pub nft_metadata_account: Option<&'a T>,

    #[cons(writable)]
    /// Optional NFT mint record to keep track of how many domains were created with this NFT
    pub nft_mint_record: Option<&'a T>,
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
            fee_account: next_account_info(accounts_iter)?,
            fee_source: next_account_info(accounts_iter)?,
            registrar: next_account_info(accounts_iter)?,
            parent_domain_account: next_account_info(accounts_iter)?,
            sub_domain_account: next_account_info(accounts_iter)?,
            sub_reverse_account: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            bonfida_fee_account: next_account_info(accounts_iter)?,
            sub_record: next_account_info(accounts_iter)?,
            nft_account: next_account_info(accounts_iter).ok(),
            nft_metadata_account: next_account_info(accounts_iter).ok(),
            nft_mint_record: next_account_info(accounts_iter).ok(),
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
        check_account_owner(accounts.fee_account, &spl_token::ID)?;
        check_account_owner(accounts.fee_source, &spl_token::ID)?;
        check_account_owner(accounts.registrar, program_id)?;
        check_account_owner(accounts.parent_domain_account, &spl_name_service::ID)?;
        check_account_owner(accounts.sub_domain_account, &system_program::ID)?;
        check_account_owner(accounts.sub_reverse_account, &system_program::ID).or_else(|_| {
            check_account_owner(accounts.sub_reverse_account, &spl_name_service::ID)
        })?;
        check_account_owner(accounts.bonfida_fee_account, &spl_token::ID)?;
        check_account_owner(accounts.sub_record, &system_program::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let (subrecord_key, subrecord_nonce) =
        SubRecord::find_key(accounts.sub_domain_account.key, program_id);
    let mut registrar = Registrar::from_account_info(accounts.registrar, Tag::Registrar)?;

    check_account_key(accounts.fee_account, &registrar.fee_account)?;
    check_account_key(accounts.parent_domain_account, &registrar.domain_account)?;
    check_account_key(accounts.sub_record, &subrecord_key)?;
    check_token_account_owner(accounts.bonfida_fee_account, &FEE_ACC_OWNER)?;

    if !params.domain.starts_with('\0') {
        return Err(SubRegisterError::InvalidSubdomain.into());
    }

    if params.domain.trim().to_lowercase() != params.domain {
        return Err(SubRegisterError::InvalidSubdomain.into());
    }

    if params.domain.contains('.') {
        return Err(SubRegisterError::InvalidSubdomain.into());
    }

    // Handle NFT gated case firts
    let mut mint_record_key: Option<Pubkey> = None;
    if let Some(collection) = registrar.nft_gated_collection {
        let nft_account = accounts
            .nft_account
            .ok_or(SubRegisterError::MustProvideNft)?;
        let nft_metadata_account = accounts
            .nft_metadata_account
            .ok_or(SubRegisterError::MustProvideNftMetadata)?;
        let nft_mint_record = accounts
            .nft_mint_record
            .ok_or(SubRegisterError::MustProvideNftMintRecord)?;

        // Accounts checks
        check_account_owner(nft_account, &spl_token::ID).unwrap();
        check_account_owner(nft_metadata_account, &mpl_token_metadata::ID).unwrap();

        let mint = check_nft_holding_and_get_mint(nft_account, accounts.fee_payer.key)?;
        check_metadata(nft_metadata_account, &collection)?;

        // Check metadata PDA deriation
        let (pda, _) = Metadata::find_pda(&mint);
        check_account_key(nft_metadata_account, &pda)?;

        // Check NFT record mint
        let (pda, nonce) = MintRecord::find_key(&mint, accounts.registrar.key, program_id);
        mint_record_key = Some(pda);
        check_account_key(nft_mint_record, &pda)?;
        let mut mint_record = if nft_mint_record.data_is_empty() {
            let mint_record = MintRecord::new(&mint);
            let seeds: &[&[u8]] = &[
                MintRecord::SEEDS,
                &accounts.registrar.key.to_bytes(),
                &mint.to_bytes(),
                &[nonce],
            ];
            Cpi::create_account(
                program_id,
                accounts.system_program,
                accounts.fee_payer,
                nft_mint_record,
                seeds,
                mint_record.borsh_len(),
            )?;
            mint_record
        } else {
            MintRecord::from_account_info(nft_mint_record, Tag::MintRecord)?
        };

        if mint_record.count >= registrar.max_nft_mint {
            return Err(SubRegisterError::MintLimitReached.into());
        }
        mint_record.count = mint_record
            .count
            .checked_add(1)
            .ok_or(SubRegisterError::Overflow)?;
        mint_record.save(&mut nft_mint_record.data.borrow_mut());
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

    // Transfer fees
    let price = utils::get_domain_price(params.domain.clone(), &registrar.price_schedule);
    let fees = (price
        .checked_mul(FEE_PCT)
        .ok_or(SubRegisterError::Overflow)?)
        / 100;
    let price = price.checked_sub(fees).ok_or(SubRegisterError::Overflow)?;
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
    let ix = spl_token::instruction::transfer(
        &spl_token::ID,
        accounts.fee_source.key,
        accounts.bonfida_fee_account.key,
        accounts.fee_payer.key,
        &[],
        fees,
    )?;
    invoke(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.fee_source.clone(),
            accounts.bonfida_fee_account.clone(),
            accounts.fee_payer.clone(),
        ],
    )?;

    // Create sub
    let space: u32 = 0;
    let lamports = Rent::get()?.minimum_balance(space as usize + NameRecordHeader::LEN);
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
            accounts.fee_payer.clone(),
            accounts.sub_domain_account.clone(),
            accounts.fee_payer.clone(),
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
                fee_payer: accounts.fee_payer.key,
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
                accounts.fee_payer.clone(),
                accounts.parent_domain_account.clone(),
                accounts.registrar.clone(),
            ],
            &[seeds],
        )?;
    }

    // Create subrecord account
    let mut sub_record = SubRecord::new(*accounts.registrar.key, *accounts.sub_domain_account.key);
    sub_record.mint_record = mint_record_key;
    let seeds: &[&[u8]] = &[
        SubRecord::SEEDS,
        &accounts.sub_domain_account.key.to_bytes(),
        &[subrecord_nonce],
    ];
    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
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
