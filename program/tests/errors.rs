//! Tests of things that should error
use sub_register::{
    entrypoint::process_instruction,
    instruction::{
        admin_register, close_registry, create_registry, edit_registry, register, unregister,
    },
    state::{registry::Registry, schedule::Price, FEE_ACC_OWNER, NAME_AUCTIONING},
};

use {
    borsh::BorshSerialize,
    name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
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
    use common::utils::sign_send_instructions;

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
    program_test.add_program("name_auctioning", NAME_AUCTIONING, None);

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
    let (_, nonce) = Pubkey::find_program_address(&[&NAME_AUCTIONING.to_bytes()], &NAME_AUCTIONING);
    program_test.add_account(
        name_auctioning::processor::CENTRAL_STATE,
        Account {
            lamports: 1_000_000,
            owner: NAME_AUCTIONING,
            data: vec![nonce],
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
    let (registry_key, _) = Registry::find_key(&name_key, &alice.pubkey(), &sub_register::ID);
    println!("[+] Registry key {}", registry_key);

    let ix = create_registry(
        create_registry::Accounts {
            system_program: &system_program::ID,
            registry: &registry_key,
            domain_name_account: &name_key,
            domain_owner: &alice.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registry::Params {
            mint,
            fee_account: *alice_fee_account,
            authority: alice.pubkey(),
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
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    // Bob registers a subdomain
    let sub_domain = "1⛽️".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);

    // To unregister later
    let sub_domain_key_to_unreg = sub_domain_key.clone();

    // Bob registers a subdomain of length 2
    let ix = register(
        register::Accounts {
            name_auctioning_program: &NAME_AUCTIONING,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &name_auctioning::processor::CENTRAL_STATE,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registry: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account: &bonfida_fee_account,
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
    ////////////////////////////////

    // Test: Non .sol domain for registry
    let (fake_registry_key, _) =
        Registry::find_key(&fake_name_key, &alice.pubkey(), &sub_register::ID);
    println!("[+] Fake registry key {}", fake_registry_key);

    let ix = create_registry(
        create_registry::Accounts {
            system_program: &system_program::ID,
            registry: &fake_registry_key,
            domain_name_account: &fake_name_key,
            domain_owner: &alice.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registry::Params {
            mint,
            fee_account: *alice_fee_account,
            authority: alice.pubkey(),
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
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &fake_authority.pubkey(),
            registry: &registry_key,
        },
        edit_registry::Params {
            new_authority: None,
            new_mint: None,
            new_fee_account: None,
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
    let ix = close_registry(
        close_registry::Accounts {
            system_program: &system_program::ID,
            registry: &registry_key,
            domain_name_account: &name_key,
            new_domain_owner: &bob.pubkey(),
            lamports_target: &mint_authority.pubkey(),
            registry_authority: &alice.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        close_registry::Params {},
    );
    let result = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
    assert!(result.is_err());

    // Test: Close registry with wrong authority
    let fake_authority = Keypair::new();
    let ix = close_registry(
        close_registry::Accounts {
            system_program: &system_program::ID,
            registry: &registry_key,
            domain_name_account: &name_key,
            new_domain_owner: &bob.pubkey(),
            lamports_target: &mint_authority.pubkey(),
            registry_authority: &fake_authority.pubkey(), // <- Fake authority and same signer
            spl_name_program_id: &spl_name_service::ID,
        },
        close_registry::Params {},
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

    let sub_domain = "this-is-an-error-test".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let ix = register(
        register::Accounts {
            name_auctioning_program: &NAME_AUCTIONING,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &name_auctioning::processor::CENTRAL_STATE,
            fee_account: alice_fee_account,
            fee_source: &bob_ata_fake_mint,
            registry: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account: &bonfida_fee_account,
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
            name_auctioning_program: &NAME_AUCTIONING,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &name_auctioning::processor::CENTRAL_STATE,
            fee_account: alice_fee_account,
            fee_source: &bob_ata,
            registry: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account: &bonfida_fee_account,
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    let result = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(result.is_err());

    // Test: Close + Register in same transaction
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
            registry: &registry_key,
            sub_domain_account: &&sub_domain_key_to_unreg,
            domain_owner: &bob.pubkey(),
        },
        unregister::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    let sub_domain = "close-register-test".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![
            close_registry(
                close_registry::Accounts {
                    system_program: &system_program::ID,
                    registry: &registry_key,
                    domain_name_account: &name_key,
                    new_domain_owner: &bob.pubkey(),
                    lamports_target: &mint_authority.pubkey(),
                    registry_authority: &alice.pubkey(),
                    spl_name_program_id: &spl_name_service::ID,
                },
                close_registry::Params {},
            ),
            register(
                register::Accounts {
                    name_auctioning_program: &NAME_AUCTIONING,
                    system_program: &system_program::ID,
                    spl_token_program: &spl_token::ID,
                    spl_name_service: &spl_name_service::ID,
                    rent_sysvar: &sysvar::rent::id(),
                    root_domain: &name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
                    reverse_lookup_class: &name_auctioning::processor::CENTRAL_STATE,
                    fee_account: alice_fee_account,
                    fee_source: &bob_ata,
                    registry: &registry_key,
                    parent_domain_account: &name_key,
                    sub_domain_account: &sub_domain_key,
                    sub_reverse_account: &sub_reverse_key,
                    fee_payer: &bob.pubkey(),
                    bonfida_fee_account: &bonfida_fee_account,
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
    let sub_domain = "invalid-sub".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![register(
            register::Accounts {
                name_auctioning_program: &NAME_AUCTIONING,
                system_program: &system_program::ID,
                spl_token_program: &spl_token::ID,
                spl_name_service: &spl_name_service::ID,
                rent_sysvar: &sysvar::rent::id(),
                root_domain: &name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
                reverse_lookup_class: &name_auctioning::processor::CENTRAL_STATE,
                fee_account: alice_fee_account,
                fee_source: &bob_ata,
                registry: &registry_key,
                parent_domain_account: &name_key,
                sub_domain_account: &sub_domain_key,
                sub_reverse_account: &sub_reverse_key,
                fee_payer: &bob.pubkey(),
                bonfida_fee_account: &bonfida_fee_account,
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
                registry: &registry_key,
                sub_domain_account: &fake_subdomain_key,
                domain_owner: &bob.pubkey(),
            },
            unregister::Params {},
        )],
        vec![&bob],
    )
    .await;
    assert!(result.is_err());

    // Test: Admin register with wrong authority
    let sub_domain = "some-admin-test".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![admin_register(
            admin_register::Accounts {
                name_auctioning_program: &NAME_AUCTIONING,
                system_program: &system_program::ID,
                spl_token_program: &spl_token::ID,
                spl_name_service: &spl_name_service::ID,
                rent_sysvar: &sysvar::rent::id(),
                root_domain: &name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
                reverse_lookup_class: &name_auctioning::processor::CENTRAL_STATE,
                registry: &registry_key,
                parent_domain_account: &name_key,
                sub_domain_account: &sub_domain_key,
                sub_reverse_account: &sub_reverse_key,
                authority: &bob.pubkey(),
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
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let result = sign_send_instructions(
        &mut prg_test_ctx,
        vec![register(
            register::Accounts {
                name_auctioning_program: &NAME_AUCTIONING,
                system_program: &system_program::ID,
                spl_token_program: &spl_token::ID,
                spl_name_service: &spl_name_service::ID,
                rent_sysvar: &sysvar::rent::id(),
                root_domain: &name_auctioning::processor::ROOT_DOMAIN_ACCOUNT,
                reverse_lookup_class: &name_auctioning::processor::CENTRAL_STATE,
                fee_account: alice_fee_account,
                fee_source: &bob_ata,
                registry: &registry_key,
                parent_domain_account: &name_key,
                sub_domain_account: &sub_domain_key,
                sub_reverse_account: &sub_reverse_key,
                fee_payer: &bob.pubkey(),
                bonfida_fee_account: &alice_fee_account,
            },
            register::Params {
                domain: format!("\0{}", sub_domain),
            },
        )],
        vec![&bob],
    )
    .await;
    assert!(result.is_err());
}
