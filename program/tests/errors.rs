//! Tests of things that should error
use solana_program::program_pack::Pack;
use sub_register::{
    entrypoint::process_instruction,
    instruction::{
        admin_register, admin_revoke, close_registrar, create_registrar, delete_subdomain_record,
        edit_registrar, nft_owner_revoke, register, unregister,
    },
    state::{
        mint_record::MintRecord, registry::Registrar, schedule::Price,
        subdomain_record::SubDomainRecord, FEE_ACC_OWNER, ROOT_DOMAIN_ACCOUNT,
    },
};
use {
    borsh::BorshSerialize,
    solana_program::{system_program, sysvar},
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{
        account::Account,
        pubkey::Pubkey,
        signer::{keypair::Keypair, Signer},
    },
    spl_associated_token_account::get_associated_token_address,
    spl_associated_token_account::instruction::create_associated_token_account,
};

pub mod common;

#[tokio::test]
async fn test_errors() {
    // Create program and test environment
    use common::utils::{random_string, sign_send_instructions};

    // Alice owns a .sol and creates the registry
    let alice = Keypair::new();

    // Bob creates a sub
    let bob = Keypair::new();
    let mint_authority = Keypair::new();

    println!("[+] Alice key {}", alice.pubkey());
    println!("[+] Bob key {}", bob.pubkey());

    let mut program_test = ProgramTest::new(
        "sub_register",
        sub_register::ID,
        processor!(process_instruction),
    );

    program_test.add_program("spl_name_service", spl_name_service::ID, None);
    program_test.add_program("sns_registrar", sns_registrar::ID, None);
    program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);

    // Add mock NFT & collection
    let mut data: Vec<u8> = vec![];
    let mut metadata = common::metadata::get_metadata();
    metadata.collection.as_mut().unwrap().verified = false;
    metadata.serialize(&mut data).unwrap();
    program_test.add_account(
        common::metadata::NFT_METADATA_KEY,
        Account {
            owner: mpl_token_metadata::ID,
            lamports: 100_000_000_000,
            data,
            ..Account::default()
        },
    );

    let mut data = [0; spl_token::state::Account::LEN];
    let mut acc_data = common::metadata::get_nft_account(&bob.pubkey());
    acc_data.amount = 0;
    acc_data.pack_into_slice(&mut data);
    let bob_nft_account_zero_amount = Pubkey::new_unique();
    program_test.add_account(
        bob_nft_account_zero_amount,
        Account {
            owner: spl_token::ID,
            lamports: 100_000_000_000,
            data: data.into(),
            ..Account::default()
        },
    );

    let mut data = [0; spl_token::state::Account::LEN];
    let acc_data = common::metadata::get_nft_account(&bob.pubkey());
    acc_data.pack_into_slice(&mut data);
    let bob_nft_account = Pubkey::new_unique();
    program_test.add_account(
        bob_nft_account,
        Account {
            owner: spl_token::ID,
            lamports: 100_000_000_000,
            data: data.into(),
            ..Account::default()
        },
    );

    program_test.add_account(
        alice.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );
    program_test.add_account(
        bob.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );

    program_test.add_account(
        sns_registrar::central_state::KEY,
        Account {
            lamports: 1_000_000,
            owner: sns_registrar::ID,
            data: vec![sns_registrar::central_state::NONCE],
            ..Account::default()
        },
    );
    program_test.add_account(
        ROOT_DOMAIN_ACCOUNT,
        Account {
            lamports: 1_000_000,
            owner: spl_name_service::ID,
            ..Account::default()
        },
    );

    // Create mock .sol domain
    let name_key = Keypair::new().pubkey();
    println!("[+] Domain name key {}", name_key);

    let root_domain_data = spl_name_service::state::NameRecordHeader {
        parent_name: ROOT_DOMAIN_ACCOUNT,
        owner: alice.pubkey(),
        class: Pubkey::default(),
    }
    .try_to_vec()
    .unwrap();
    program_test.add_account(
        name_key,
        Account {
            lamports: 1_000_000,
            data: root_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    // Create fake domain name
    let fake_name_key = Keypair::new().pubkey();
    println!("[+] Fake domain name key {}", fake_name_key);

    let fake_domain_data = spl_name_service::state::NameRecordHeader {
        parent_name: Pubkey::new_unique(),
        owner: alice.pubkey(),
        class: Pubkey::default(),
    }
    .try_to_vec()
    .unwrap();
    program_test.add_account(
        fake_name_key,
        Account {
            lamports: 1_000_000,
            data: fake_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    // Domain owned by Bob
    let fake_subdomain_key = Keypair::new().pubkey();
    println!("[+] Fake subdomain name key {}", fake_name_key);

    let fake_domain_data = spl_name_service::state::NameRecordHeader {
        parent_name: Pubkey::new_unique(),
        owner: bob.pubkey(),
        class: Pubkey::default(),
    }
    .try_to_vec()
    .unwrap();
    program_test.add_account(
        fake_subdomain_key,
        Account {
            lamports: 1_000_000,
            data: fake_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    //
    // Create mint
    //
    let (mint, _) =
        common::utils::mint_bootstrap(None, 6, &mut program_test, &mint_authority.pubkey());
    let (fake_mint, _) =
        common::utils::mint_bootstrap(None, 6, &mut program_test, &mint_authority.pubkey());

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    // Create ATA for Bob and mint tokens into it
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &bob.pubkey(),
        &mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let bob_ata = get_associated_token_address(&bob.pubkey(), &mint);
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &bob_ata,
        &mint_authority.pubkey(),
        &[],
        10_000_000_000,
    )
    .unwrap();

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&mint_authority])
        .await
        .unwrap();

    // Creates fee account for Alice
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &alice.pubkey(),
        &mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let alice_fee_account = &get_associated_token_address(&alice.pubkey(), &mint);

    // Creates Bonfida fee account
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &FEE_ACC_OWNER,
        &mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let bonfida_fee_account = &get_associated_token_address(&FEE_ACC_OWNER, &mint);

    // Alice creates regis&try
    let (registry_key, _) = Registrar::find_key(&name_key, &sub_register::ID);
    println!("[+] Registry key {}", registry_key);

    let ix = create_registrar(
        create_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &registry_key,
            domain_name_account: &name_key,
            domain_owner: &alice.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registrar::Params {
            mint,
            fee_account: *alice_fee_account,
            authority: alice.pubkey(),
            allow_revoke: false,
            max_nft_mint: 0,
            price_schedule: vec![
                Price {
                    length: 1,
                    price: 10_000_000,
                },
                Price {
                    length: 2,
                    price: 8_000_000,
                },
                Price {
                    length: 3,
                    price: 7_000_000,
                },
                Price {
                    length: 4,
                    price: 6_000_000,
                },
                Price {
                    length: 5,
                    price: 10_000_000_001,
                },
            ],
            nft_gated_collection: None,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    // Bob registers a subdomain
    let mut sub_domain = random_string();
    sub_domain.truncate(2);
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);

    // To unregister later
    let sub_domain_key_to_unreg = sub_domain_key;

    // Bob registers a subdomain of length 2
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: None,
            nft_metadata_account: None,
            sub_record: &subrecord_key,
            nft_mint_record: None,
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    ////////////////////////////////
    //
    // Test several things that should error
    // - Non .sol domain for registry
    // - Wrong authority edit registry
    // - Close registry with != 0 total registered
    // - Close registry with wrong authority
    // - Register with wrong mint
    // - Register with not enough funds
    // - Close + Register in same transaction
    // - Invalid sub
    // - Unregister invalid sub
    // - Admin register with wrong authority
    // - Wrong bonfida fee account
    // - Register with 0 NFT
    // - Register with unverified collection
    // - Unregister with wrong sub record
    // - Delete subrecord passing wrong name
    // - Try to revoke in non revokable registrar
    // - Non admin revoke
    ////////////////////////////////

    // Test: Non .sol domain for registry
    let (fake_registry_key, _) = Registrar::find_key(&fake_name_key, &sub_register::ID);
    println!("[+] Fake registry key {}", fake_registry_key);

    let ix = create_registrar(
        create_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &fake_registry_key,
            domain_name_account: &fake_name_key,
            domain_owner: &alice.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registrar::Params {
            mint,
            fee_account: *alice_fee_account,
            authority: alice.pubkey(),
            nft_gated_collection: None,
            max_nft_mint: 0,
            allow_revoke: false,
            price_schedule: vec![
                Price {
                    length: 1,
                    price: 10_000_000_001,
                },
                Price {
                    length: 2,
                    price: 10_000_000_001,
                },
            ],
        },
    );
    let result = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
    assert!(result.is_err());

    // Test: Wrong authority edit registry
    let fake_authority = Keypair::new();
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &fake_authority.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_authority: None,
            new_mint: None,
            new_fee_account: None,
            new_max_nft_mint: None,
            new_price_schedule: Some(vec![
                Price {
                    length: 1,
                    price: 10_000_000,
                },
                Price {
                    length: 2,
                    price: 10_000_000,
                },
                Price {
                    length: 3,
                    price: 5_000_000,
                },
            ]),
        },
    );
    let result = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&fake_authority]).await;
    assert!(result.is_err());

    // Test: Close registry with != 0 total registered
    let ix = close_registrar(
        close_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &registry_key,
            domain_name_account: &name_key,
            new_domain_owner: &bob.pubkey(),
            lamports_target: &mint_authority.pubkey(),
            registry_authority: &alice.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        close_registrar::Params {},
    );
    let result = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
    assert!(result.is_err());

    // Test: Close registry with wrong authority
    let fake_authority = Keypair::new();
    let ix = close_registrar(
        close_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &registry_key,
            domain_name_account: &name_key,
            new_domain_owner: &bob.pubkey(),
            lamports_target: &mint_authority.pubkey(),
            registry_authority: &fake_authority.pubkey(), // <- Fake authority and same signer
            spl_name_program_id: &spl_name_service::ID,
        },
        close_registrar::Params {},
    );
    let result = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&fake_authority]).await;
    assert!(result.is_err());

    // Test: Register with wrong mint
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &bob.pubkey(),
        &fake_mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let bob_ata_fake_mint = get_associated_token_address(&bob.pubkey(), &fake_mint);

    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata_fake_mint,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: None,
            nft_metadata_account: None,
            sub_record: &subrecord_key,
            nft_mint_record: None,
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    let result = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(result.is_err());

    // Test: Register with not enough funds
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: None,
            nft_metadata_account: None,
            sub_record: &subrecord_key,
            nft_mint_record: None,
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    let result = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(result.is_err());

    // Test: Close + Register in same transaction
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key_to_unreg, &sub_register::ID);
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &bob_ata,
        &mint_authority.pubkey(),
        &[],
        1 + 8_000_000,
    )
    .unwrap();
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&mint_authority])
        .await
        .unwrap();
    let ix = unregister(
        unregister::Accounts {
            system_program: &system_program::ID,
            spl_name_service: &spl_name_service::ID,
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key_to_unreg,
            domain_owner: &bob.pubkey(),
            sub_record: &subrecord_key,
            mint_record: None,
        },
        unregister::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key_to_unreg, &sub_register::ID);
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![
            close_registrar(
                close_registrar::Accounts {
                    system_program: &system_program::ID,
                    registrar: &registry_key,
                    domain_name_account: &name_key,
                    new_domain_owner: &bob.pubkey(),
                    lamports_target: &mint_authority.pubkey(),
                    registry_authority: &alice.pubkey(),
                    spl_name_program_id: &spl_name_service::ID,
                },
                close_registrar::Params {},
            ),
            register(
                register::Accounts {
                    sns_registrar_program: &sns_registrar::ID,
                    system_program: &system_program::ID,
                    spl_token_program: &spl_token::ID,
                    spl_name_service: &spl_name_service::ID,
                    rent_sysvar: &sysvar::rent::id(),
                    root_domain: &ROOT_DOMAIN_ACCOUNT,
                    reverse_lookup_class: &sns_registrar::central_state::KEY,
                    fee_account: alice_fee_account,
                    fee_source: &bob_ata,
                    registrar: &registry_key,
                    parent_domain_account: &name_key,
                    sub_domain_account: &sub_domain_key,
                    sub_reverse_account: &sub_reverse_key,
                    fee_payer: &bob.pubkey(),
                    bonfida_fee_account,
                    nft_account: None,
                    nft_metadata_account: None,
                    sub_record: &subrecord_key,
                    nft_mint_record: None,
                },
                register::Params {
                    domain: format!("\0{}", sub_domain),
                },
            ),
        ],
        vec![&alice, &bob],
    )
    .await;
    assert!(result.is_err());

    // Test: Invalid sub
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![register(
            register::Accounts {
                sns_registrar_program: &sns_registrar::ID,
                system_program: &system_program::ID,
                spl_token_program: &spl_token::ID,
                spl_name_service: &spl_name_service::ID,
                rent_sysvar: &sysvar::rent::id(),
                root_domain: &ROOT_DOMAIN_ACCOUNT,
                reverse_lookup_class: &sns_registrar::central_state::KEY,
                fee_account: alice_fee_account,
                fee_source: &bob_ata,
                registrar: &registry_key,
                parent_domain_account: &name_key,
                sub_domain_account: &sub_domain_key,
                sub_reverse_account: &sub_reverse_key,
                fee_payer: &bob.pubkey(),
                bonfida_fee_account,
                nft_account: None,
                nft_metadata_account: None,
                sub_record: &subrecord_key,
                nft_mint_record: None,
            },
            register::Params { domain: sub_domain },
        )],
        vec![&bob],
    )
    .await;
    assert!(result.is_err());

    // Test: Unregister invalid sub
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![unregister(
            unregister::Accounts {
                system_program: &system_program::ID,
                spl_name_service: &spl_name_service::ID,
                registrar: &registry_key,
                sub_domain_account: &fake_subdomain_key,
                domain_owner: &bob.pubkey(),
                sub_record: &subrecord_key,
                mint_record: None,
            },
            unregister::Params {},
        )],
        vec![&bob],
    )
    .await;
    assert!(result.is_err());

    // Test: Admin register with wrong authority
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![admin_register(
            admin_register::Accounts {
                sns_registrar_program: &sns_registrar::ID,
                system_program: &system_program::ID,
                spl_token_program: &spl_token::ID,
                spl_name_service: &spl_name_service::ID,
                rent_sysvar: &sysvar::rent::id(),
                root_domain: &ROOT_DOMAIN_ACCOUNT,
                reverse_lookup_class: &sns_registrar::central_state::KEY,
                registrar: &registry_key,
                parent_domain_account: &name_key,
                sub_domain_account: &sub_domain_key,
                sub_reverse_account: &sub_reverse_key,
                authority: &bob.pubkey(),
                sub_record: &subrecord_key,
            },
            admin_register::Params {
                domain: format!("\0{}", sub_domain),
            },
        )],
        vec![&bob],
    )
    .await;
    assert!(result.is_err());

    // Test: Wrong bonfida fee account
    let sub_domain = "invalid-sub".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![register(
            register::Accounts {
                sns_registrar_program: &sns_registrar::ID,
                system_program: &system_program::ID,
                spl_token_program: &spl_token::ID,
                spl_name_service: &spl_name_service::ID,
                rent_sysvar: &sysvar::rent::id(),
                root_domain: &ROOT_DOMAIN_ACCOUNT,
                reverse_lookup_class: &sns_registrar::central_state::KEY,
                fee_account: alice_fee_account,
                fee_source: &bob_ata,
                registrar: &registry_key,
                parent_domain_account: &name_key,
                sub_domain_account: &sub_domain_key,
                sub_reverse_account: &sub_reverse_key,
                fee_payer: &bob.pubkey(),
                bonfida_fee_account: alice_fee_account,
                nft_account: None,
                nft_metadata_account: None,
                sub_record: &subrecord_key,
                nft_mint_record: None,
            },
            register::Params {
                domain: format!("\0{}", sub_domain),
            },
        )],
        vec![&bob],
    )
    .await;
    assert!(result.is_err());

    ////////////////////////////////////////
    //
    // Test with NFT gated registrar
    //
    ////////////////////////////////////////

    let ix = close_registrar(
        close_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &registry_key,
            domain_name_account: &name_key,
            new_domain_owner: &alice.pubkey(),
            lamports_target: &mint_authority.pubkey(),
            registry_authority: &alice.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        close_registrar::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    let ix = create_registrar(
        create_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &registry_key,
            domain_name_account: &name_key,
            domain_owner: &alice.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registrar::Params {
            max_nft_mint: 4,
            allow_revoke: false,
            nft_gated_collection: Some(common::metadata::COLLECTION_KEY),
            mint,
            fee_account: *alice_fee_account,
            authority: alice.pubkey(),
            price_schedule: (vec![
                Price {
                    length: 1,
                    price: 10_000_000,
                },
                Price {
                    length: 2,
                    price: 10_000_000,
                },
            ]),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    // Test: Register with 0 NFT
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record, _) = MintRecord::find_key(
        &common::metadata::NFT_MINT,
        &registry_key,
        &sub_register::ID,
    );

    // Bob registers a subdomain
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account_zero_amount),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());

    // Test: Register with unverified collection
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record, _) = MintRecord::find_key(
        &common::metadata::NFT_MINT,
        &registry_key,
        &sub_register::ID,
    );
    // Bob registers a subdomain
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());

    // Test: Unregister with wrong sub record
    // First close old registrar + create new + register 2 subs
    let sub_domain_1 = random_string();
    let sub_domain_key_1 = sub_register::utils::get_subdomain_key(&sub_domain_1, &name_key);
    let sub_reverse_key_1 = sub_register::utils::get_subdomain_reverse(&sub_domain_1, &name_key);
    let (subrecord_key_1, _) = SubDomainRecord::find_key(&sub_domain_key_1, &sub_register::ID);

    let sub_domain_2 = random_string();
    let sub_domain_key_2 = sub_register::utils::get_subdomain_key(&sub_domain_2, &name_key);
    let sub_reverse_key_2 = sub_register::utils::get_subdomain_reverse(&sub_domain_2, &name_key);
    let (subrecord_key_2, _) = SubDomainRecord::find_key(&sub_domain_key_2, &sub_register::ID);
    let fee_payer = prg_test_ctx.payer.pubkey();
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![close_registrar(
            close_registrar::Accounts {
                system_program: &system_program::ID,
                registrar: &registry_key,
                domain_name_account: &name_key,
                new_domain_owner: &alice.pubkey(),
                lamports_target: &mint_authority.pubkey(),
                registry_authority: &alice.pubkey(),
                spl_name_program_id: &spl_name_service::ID,
            },
            close_registrar::Params {},
        )],
        vec![&alice],
    )
    .await
    .unwrap();
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![
            create_registrar(
                create_registrar::Accounts {
                    system_program: &system_program::ID,
                    registrar: &registry_key,
                    domain_name_account: &name_key,
                    domain_owner: &alice.pubkey(),
                    fee_payer: &fee_payer,
                    spl_name_program_id: &spl_name_service::ID,
                },
                create_registrar::Params {
                    max_nft_mint: 4,
                    allow_revoke: true,
                    nft_gated_collection: None,
                    mint,
                    fee_account: *alice_fee_account,
                    authority: alice.pubkey(),
                    price_schedule: (vec![
                        Price {
                            length: 1,
                            price: 10_000_000,
                        },
                        Price {
                            length: 2,
                            price: 10_000_000,
                        },
                    ]),
                },
            ),
            register(
                register::Accounts {
                    sns_registrar_program: &sns_registrar::ID,
                    system_program: &system_program::ID,
                    spl_token_program: &spl_token::ID,
                    spl_name_service: &spl_name_service::ID,
                    rent_sysvar: &sysvar::rent::id(),
                    root_domain: &ROOT_DOMAIN_ACCOUNT,
                    reverse_lookup_class: &sns_registrar::central_state::KEY,
                    fee_account: alice_fee_account,
                    fee_source: &bob_ata,
                    registrar: &registry_key,
                    parent_domain_account: &name_key,
                    sub_domain_account: &sub_domain_key_1,
                    sub_reverse_account: &sub_reverse_key_1,
                    fee_payer: &bob.pubkey(),
                    bonfida_fee_account,
                    nft_account: None,
                    nft_metadata_account: None,
                    sub_record: &subrecord_key_1,
                    nft_mint_record: Some(&mint_record),
                },
                register::Params {
                    domain: format!("\0{}", sub_domain_1),
                },
            ),
            register(
                register::Accounts {
                    sns_registrar_program: &sns_registrar::ID,
                    system_program: &system_program::ID,
                    spl_token_program: &spl_token::ID,
                    spl_name_service: &spl_name_service::ID,
                    rent_sysvar: &sysvar::rent::id(),
                    root_domain: &ROOT_DOMAIN_ACCOUNT,
                    reverse_lookup_class: &sns_registrar::central_state::KEY,
                    fee_account: alice_fee_account,
                    fee_source: &bob_ata,
                    registrar: &registry_key,
                    parent_domain_account: &name_key,
                    sub_domain_account: &sub_domain_key_2,
                    sub_reverse_account: &sub_reverse_key_2,
                    fee_payer: &bob.pubkey(),
                    bonfida_fee_account,
                    nft_account: None,
                    nft_metadata_account: None,
                    sub_record: &subrecord_key_2,
                    nft_mint_record: Some(&mint_record),
                },
                register::Params {
                    domain: format!("\0{}", sub_domain_2),
                },
            ),
        ],
        vec![&bob, &alice],
    )
    .await
    .unwrap();

    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![unregister(
            unregister::Accounts {
                system_program: &system_program::ID,
                spl_name_service: &spl_name_service::ID,
                registrar: &registry_key,
                sub_domain_account: &sub_domain_key_1,
                domain_owner: &bob.pubkey(),
                sub_record: &subrecord_key_2,
                mint_record: Some(&mint_record),
            },
            unregister::Params {},
        )],
        vec![&bob],
    )
    .await;
    assert!(result.is_err());

    // Test: Delete subrecord passing wrong name
    // Delete domain via SNS
    let ix = spl_name_service::instruction::delete(
        spl_name_service::ID,
        sub_domain_key_1,
        bob.pubkey(),
        bob.pubkey(),
    )
    .unwrap();
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let ix = delete_subdomain_record(
        delete_subdomain_record::Accounts {
            sub_domain: &Pubkey::new_unique(),
            lamports_target: &bob.pubkey(),
            sub_record: &SubDomainRecord::find_key(&sub_domain_key_1, &sub_register::ID).0,
            mint_record: Some(&mint_record),
            registrar: &registry_key,
        },
        delete_subdomain_record::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![]).await;
    assert!(res.is_err());

    // Clean up
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![
            delete_subdomain_record(
                delete_subdomain_record::Accounts {
                    sub_domain: &sub_domain_key_1,
                    lamports_target: &bob.pubkey(),
                    sub_record: &SubDomainRecord::find_key(&sub_domain_key_1, &sub_register::ID).0,
                    mint_record: None,
                    registrar: &registry_key,
                },
                delete_subdomain_record::Params {},
            ),
            unregister(
                unregister::Accounts {
                    system_program: &system_program::ID,
                    spl_name_service: &spl_name_service::ID,
                    registrar: &registry_key,
                    sub_domain_account: &sub_domain_key_2,
                    domain_owner: &bob.pubkey(),
                    sub_record: &subrecord_key_2,
                    mint_record: None,
                },
                unregister::Params {},
            ),
            close_registrar(
                close_registrar::Accounts {
                    system_program: &system_program::ID,
                    registrar: &registry_key,
                    domain_name_account: &name_key,
                    new_domain_owner: &alice.pubkey(),
                    lamports_target: &mint_authority.pubkey(),
                    registry_authority: &alice.pubkey(),
                    spl_name_program_id: &spl_name_service::ID,
                },
                close_registrar::Params {},
            ),
        ],
        vec![&alice, &bob],
    )
    .await
    .unwrap();

    // Test: Try to revoke in non revokable registrar
    let ix = create_registrar(
        create_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &registry_key,
            domain_name_account: &name_key,
            domain_owner: &alice.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registrar::Params {
            max_nft_mint: 4,
            allow_revoke: false,
            nft_gated_collection: None,
            mint,
            fee_account: *alice_fee_account,
            authority: alice.pubkey(),
            price_schedule: (vec![
                Price {
                    length: 1,
                    price: 10_000_000,
                },
                Price {
                    length: 2,
                    price: 10_000_000,
                },
            ]),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);

    // Bob registers a subdomain
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: None,
            nft_metadata_account: None,
            sub_record: &subrecord_key,
            nft_mint_record: None,
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let ix = admin_revoke(
        admin_revoke::Accounts {
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            sub_record: &subrecord_key,
            sub_owner: &bob.pubkey(),
            parent_domain: &name_key,
            authority: &alice.pubkey(),
            name_class: &Pubkey::default(),
            spl_name_service: &spl_name_service::ID,
            mint_record: None,
        },
        admin_revoke::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
    assert!(res.is_err());

    // Clean up
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![
            unregister(
                unregister::Accounts {
                    system_program: &system_program::ID,
                    spl_name_service: &spl_name_service::ID,
                    registrar: &registry_key,
                    sub_domain_account: &sub_domain_key,
                    domain_owner: &bob.pubkey(),
                    sub_record: &subrecord_key,
                    mint_record: None,
                },
                unregister::Params {},
            ),
            close_registrar(
                close_registrar::Accounts {
                    system_program: &system_program::ID,
                    registrar: &registry_key,
                    domain_name_account: &name_key,
                    new_domain_owner: &alice.pubkey(),
                    lamports_target: &mint_authority.pubkey(),
                    registry_authority: &alice.pubkey(),
                    spl_name_program_id: &spl_name_service::ID,
                },
                close_registrar::Params {},
            ),
        ],
        vec![&alice, &bob],
    )
    .await
    .unwrap();

    // Test: Non admin revoke
    let ix = create_registrar(
        create_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &registry_key,
            domain_name_account: &name_key,
            domain_owner: &alice.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registrar::Params {
            max_nft_mint: 4,
            allow_revoke: true,
            nft_gated_collection: None,
            mint,
            fee_account: *alice_fee_account,
            authority: alice.pubkey(),
            price_schedule: (vec![
                Price {
                    length: 1,
                    price: 10_000_000,
                },
                Price {
                    length: 2,
                    price: 10_000_000,
                },
            ]),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);

    // Bob registers a subdomain
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: None,
            nft_metadata_account: None,
            sub_record: &subrecord_key,
            nft_mint_record: None,
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let ix = admin_revoke(
        admin_revoke::Accounts {
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            sub_record: &subrecord_key,
            sub_owner: &bob.pubkey(),
            parent_domain: &name_key,
            authority: &bob.pubkey(),
            name_class: &Pubkey::default(),
            spl_name_service: &spl_name_service::ID,
            mint_record: None,
        },
        admin_revoke::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_errors_nft() {
    // Create program and test environment
    use common::utils::{random_string, sign_send_instructions};

    // Alice owns a .sol and creates the registry
    let alice = Keypair::new();

    // Bob creates a sub
    let bob = Keypair::new();
    let mint_authority = Keypair::new();

    println!("[+] Alice key {}", alice.pubkey());
    println!("[+] Bob key {}", bob.pubkey());

    let mut program_test = ProgramTest::new(
        "sub_register",
        sub_register::ID,
        processor!(process_instruction),
    );

    program_test.add_program("spl_name_service", spl_name_service::ID, None);
    program_test.add_program("sns_registrar", sns_registrar::ID, None);
    program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);

    // Add mock NFT & collection
    let mut data: Vec<u8> = vec![];
    let metadata = common::metadata::get_metadata();
    metadata.serialize(&mut data).unwrap();
    program_test.add_account(
        common::metadata::NFT_METADATA_KEY,
        Account {
            owner: mpl_token_metadata::ID,
            lamports: 100_000_000_000,
            data,
            ..Account::default()
        },
    );

    let mut data = [0; spl_token::state::Account::LEN];
    let acc_data = common::metadata::get_nft_account(&bob.pubkey());
    acc_data.pack_into_slice(&mut data);
    let bob_nft_account = Pubkey::new_unique();
    program_test.add_account(
        bob_nft_account,
        Account {
            owner: spl_token::ID,
            lamports: 100_000_000_000,
            data: data.into(),
            ..Account::default()
        },
    );

    program_test.add_account(
        alice.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );
    program_test.add_account(
        bob.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );

    program_test.add_account(
        sns_registrar::central_state::KEY,
        Account {
            lamports: 1_000_000,
            owner: sns_registrar::ID,
            data: vec![sns_registrar::central_state::NONCE],
            ..Account::default()
        },
    );
    program_test.add_account(
        ROOT_DOMAIN_ACCOUNT,
        Account {
            lamports: 1_000_000,
            owner: spl_name_service::ID,
            ..Account::default()
        },
    );

    // Create mock .sol domain
    let name_key = Keypair::new().pubkey();
    println!("[+] Domain name key {}", name_key);

    let root_domain_data = spl_name_service::state::NameRecordHeader {
        parent_name: ROOT_DOMAIN_ACCOUNT,
        owner: alice.pubkey(),
        class: Pubkey::default(),
    }
    .try_to_vec()
    .unwrap();
    program_test.add_account(
        name_key,
        Account {
            lamports: 1_000_000,
            data: root_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    //
    // Create mint
    //
    let (mint, _) =
        common::utils::mint_bootstrap(None, 6, &mut program_test, &mint_authority.pubkey());

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    // Create ATA for Bob and mint tokens into it
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &bob.pubkey(),
        &mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let bob_ata = get_associated_token_address(&bob.pubkey(), &mint);
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &bob_ata,
        &mint_authority.pubkey(),
        &[],
        100_000_000_000_000,
    )
    .unwrap();

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&mint_authority])
        .await
        .unwrap();

    // Creates fee account for Alice
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &alice.pubkey(),
        &mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let alice_fee_account = &get_associated_token_address(&alice.pubkey(), &mint);

    // Creates Bonfida fee account
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &FEE_ACC_OWNER,
        &mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let bonfida_fee_account = &get_associated_token_address(&FEE_ACC_OWNER, &mint);

    // Alice creates regis&try
    let (registry_key, _) = Registrar::find_key(&name_key, &sub_register::ID);
    println!("[+] Registry key {}", registry_key);

    let ix = create_registrar(
        create_registrar::Accounts {
            system_program: &system_program::ID,
            registrar: &registry_key,
            domain_name_account: &name_key,
            domain_owner: &alice.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registrar::Params {
            mint,
            fee_account: *alice_fee_account,
            authority: alice.pubkey(),
            allow_revoke: true,
            max_nft_mint: 4,
            price_schedule: (vec![
                Price {
                    length: 1,
                    price: 10_000_000,
                },
                Price {
                    length: 2,
                    price: 8_000_000,
                },
                Price {
                    length: 3,
                    price: 7_000_000,
                },
                Price {
                    length: 4,
                    price: 6_000_000,
                },
                Price {
                    length: 5,
                    price: 10_000_000_001,
                },
            ]),
            nft_gated_collection: Some(common::metadata::COLLECTION_KEY),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    let mut sub_to_revoke = Pubkey::default();
    for _ in 0..4 {
        let sub_domain = random_string();
        let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
        let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
        let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
        let (mint_record, _) = MintRecord::find_key(
            &common::metadata::NFT_MINT,
            &registry_key,
            &sub_register::ID,
        );
        sub_to_revoke = sub_domain_key;

        // Bob registers a subdomain
        let ix = register(
            register::Accounts {
                sns_registrar_program: &sns_registrar::ID,
                system_program: &system_program::ID,
                spl_token_program: &spl_token::ID,
                spl_name_service: &spl_name_service::ID,
                rent_sysvar: &sysvar::rent::id(),
                root_domain: &ROOT_DOMAIN_ACCOUNT,
                reverse_lookup_class: &sns_registrar::central_state::KEY,
                fee_account: alice_fee_account,
                fee_source: &bob_ata,
                registrar: &registry_key,
                parent_domain_account: &name_key,
                sub_domain_account: &sub_domain_key,
                sub_reverse_account: &sub_reverse_key,
                fee_payer: &bob.pubkey(),
                bonfida_fee_account,
                nft_account: Some(&bob_nft_account),
                nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
                sub_record: &subrecord_key,
                nft_mint_record: Some(&mint_record),
            },
            register::Params {
                domain: format!("\0{}", sub_domain),
            },
        );
        sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
            .await
            .unwrap();
    }

    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record, _) = MintRecord::find_key(
        &common::metadata::NFT_MINT,
        &registry_key,
        &sub_register::ID,
    );

    // Bob registers a subdomain
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());

    // Alice tries to revoke domain from Bob through `nft_owner_revoke`
    let ix = nft_owner_revoke(
        nft_owner_revoke::Accounts {
            registrar: &registry_key,
            sub_domain_account: &sub_to_revoke,
            sub_record: &SubDomainRecord::find_key(&sub_to_revoke, &sub_register::ID).0,
            sub_owner: &bob.pubkey(),
            parent_domain: &name_key,
            nft_account: &bob_nft_account,
            nft_metadata: &common::metadata::NFT_METADATA_KEY,
            nft_owner: &alice.pubkey(),
            name_class: &Pubkey::default(),
            nft_mint_record: &mint_record,
            spl_name_service: &spl_name_service::ID,
        },
        nft_owner_revoke::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
    assert!(res.is_err());

    // Transfer domain to Alice
    let ix = spl_name_service::instruction::transfer(
        spl_name_service::ID,
        alice.pubkey(),
        sub_to_revoke,
        bob.pubkey(),
        None,
    )
    .unwrap();
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    // Bob tries a successful revoke via NFT
    let ix = nft_owner_revoke(
        nft_owner_revoke::Accounts {
            registrar: &registry_key,
            sub_domain_account: &sub_to_revoke,
            sub_record: &SubDomainRecord::find_key(&sub_to_revoke, &sub_register::ID).0,
            sub_owner: &bob.pubkey(),
            parent_domain: &name_key,
            nft_account: &bob_nft_account,
            nft_metadata: &common::metadata::NFT_METADATA_KEY,
            nft_owner: &bob.pubkey(),
            name_class: &Pubkey::default(),
            nft_mint_record: &mint_record,
            spl_name_service: &spl_name_service::ID,
        },
        nft_owner_revoke::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    // After this Bob should be able to register a new domain
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record, _) = MintRecord::find_key(
        &common::metadata::NFT_MINT,
        &registry_key,
        &sub_register::ID,
    );

    // Bob registers a subdomain
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    // Alice revokes via admin
    let ix = admin_revoke(
        admin_revoke::Accounts {
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            sub_record: &subrecord_key,
            parent_domain: &name_key,
            sub_owner: &bob.pubkey(),
            authority: &alice.pubkey(),
            spl_name_service: &spl_name_service::ID,
            name_class: &Pubkey::default(),
            mint_record: Some(&mint_record),
        },
        admin_revoke::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    // After this Bob should be able to register a new domain
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let (subrecord_key, _) = SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record, _) = MintRecord::find_key(
        &common::metadata::NFT_MINT,
        &registry_key,
        &sub_register::ID,
    );

    // Bob registers a subdomain
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
}
