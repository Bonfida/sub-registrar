//! Tests of state integrity

use solana_program::program_pack::Pack;
use sub_register::{
    entrypoint::process_instruction,
    instruction::{close_registry, create_registry, edit_registry, register, unregister},
    state::{registry::Registry, schedule::Price, Tag},
    utils::get_subdomain_key,
};

use {
    borsh::{BorshDeserialize, BorshSerialize},
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
async fn test_state() {
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
    program_test.add_program(
        "name_auctioning",
        sub_register::instruction::register::NAME_AUCTIONING,
        None,
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
    let (_, nonce) = Pubkey::find_program_address(
        &[&sub_register::instruction::register::NAME_AUCTIONING.to_bytes()],
        &sub_register::instruction::register::NAME_AUCTIONING,
    );
    program_test.add_account(
        name_auctioning::processor::CENTRAL_STATE,
        Account {
            lamports: 1_000_000,
            owner: sub_register::instruction::register::NAME_AUCTIONING,
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

    // A&lice creates regis&try
    let (registry_key, nonce) = Registry::find_key(&name_key, &alice.pubkey(), &sub_register::ID);
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
                    length: 2,
                    price: 10_000_000,
                },
                Price {
                    length: 1,
                    price: 10_000_000,
                },
            ],
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    let mut expected_registry = Registry {
        tag: Tag::Registry,
        nonce,
        authority: alice.pubkey(),
        fee_account: *alice_fee_account,
        mint,
        domain_account: name_key,
        total_sub_created: 0,
        price_schedule: vec![
            Price {
                length: 1,
                price: 10_000_000,
            },
            Price {
                length: 2,
                price: 10_000_000,
            },
        ],
    };
    assert_eq!(registry, expected_registry);

    // Increase vec size
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
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
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.price_schedule = vec![
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
    ];
    assert_eq!(registry, expected_registry);

    // Decrease vec size
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
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
                    price: 8_000_000,
                },
            ]),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.price_schedule = vec![
        Price {
            length: 1,
            price: 10_000_000,
        },
        Price {
            length: 2,
            price: 8_000_000,
        },
    ];
    assert_eq!(registry, expected_registry);

    // Change mint
    let new_mint = Pubkey::new_unique();
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registry: &registry_key,
        },
        edit_registry::Params {
            new_authority: None,
            new_mint: Some(new_mint),
            new_fee_account: None,
            new_price_schedule: None,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.mint = new_mint;
    assert_eq!(registry, expected_registry);

    // Change mint back so we don't have to create a new token
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registry: &registry_key,
        },
        edit_registry::Params {
            new_authority: None,
            new_mint: Some(mint),
            new_fee_account: None,
            new_price_schedule: None,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.mint = mint;
    assert_eq!(registry, expected_registry);

    // Change fee account
    let new_fee_account = Pubkey::new_unique();
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registry: &registry_key,
        },
        edit_registry::Params {
            new_authority: None,
            new_mint: None,
            new_fee_account: Some(new_fee_account),
            new_price_schedule: None,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.fee_account = new_fee_account;
    assert_eq!(registry, expected_registry);
    // Change it back
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registry: &registry_key,
        },
        edit_registry::Params {
            new_authority: None,
            new_mint: None,
            new_fee_account: Some(*alice_fee_account),
            new_price_schedule: None,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.fee_account = *alice_fee_account;
    assert_eq!(registry, expected_registry);

    // Change authority
    let new_authority = Keypair::new();
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registry: &registry_key,
        },
        edit_registry::Params {
            new_authority: Some(new_authority.pubkey()),
            new_mint: None,
            new_fee_account: None,
            new_price_schedule: None,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.authority = new_authority.pubkey();
    assert_eq!(registry, expected_registry);

    // Change authority back to alice
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &new_authority.pubkey(),
            registry: &registry_key,
        },
        edit_registry::Params {
            new_authority: Some(alice.pubkey()),
            new_mint: None,
            new_fee_account: None,
            new_price_schedule: None,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&new_authority])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.authority = alice.pubkey();
    assert_eq!(registry, expected_registry);

    let sub_domain = "some-test".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);

    // Bob registers a subdomain
    let ix = register(
        register::Accounts {
            name_auctioning_program: &register::NAME_AUCTIONING,
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
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.total_sub_created = 1;
    assert_eq!(registry, expected_registry);

    let ix = unregister(
        unregister::Accounts {
            system_program: &system_program::ID,
            spl_name_service: &spl_name_service::ID,
            registry: &registry_key,
            sub_domain_account: &sub_domain_key,
            domain_owner: &bob.pubkey(),
        },
        unregister::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.total_sub_created = 0;
    assert_eq!(registry, expected_registry);

    // Verify fees received
    let acc = prg_test_ctx
        .banks_client
        .get_account(*alice_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    assert_eq!(token_account.amount, 8_000_000);

    // Change price schedule + register + verify
    let ix = edit_registry(
        edit_registry::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registry: &registry_key,
        },
        edit_registry::Params {
            new_authority: None,
            new_mint: None,
            new_fee_account: None,
            new_price_schedule: Some(vec![
                // Not in the right order in purpose
                Price {
                    length: 2,
                    price: 8_000_000,
                },
                Price {
                    length: 4,
                    price: 6_000_000,
                },
                Price {
                    length: 1,
                    price: 10_000_000,
                },
                Price {
                    length: 3,
                    price: 7_000_000,
                },
                Price {
                    length: 5,
                    price: 5_000_000,
                },
            ]),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.price_schedule = vec![
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
            price: 5_000_000,
        },
    ];
    assert_eq!(registry, expected_registry);

    let sub_domain = "1".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);

    // Bob registers a subdomain of length 1
    let ix = register(
        register::Accounts {
            name_auctioning_program: &register::NAME_AUCTIONING,
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
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.total_sub_created = 1;
    assert_eq!(registry, expected_registry);

    // Verify fees received
    let acc = prg_test_ctx
        .banks_client
        .get_account(*alice_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    assert_eq!(token_account.amount, 8_000_000 + 10_000_000);

    let sub_domain = "1⛽️".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);

    // Bob registers a subdomain of length 2
    let ix = register(
        register::Accounts {
            name_auctioning_program: &register::NAME_AUCTIONING,
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
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.total_sub_created = 2;
    assert_eq!(registry, expected_registry);

    // Verify fees received
    let acc = prg_test_ctx
        .banks_client
        .get_account(*alice_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    assert_eq!(token_account.amount, 8_000_000 + 10_000_000 + 8_000_000);

    let sub_domain = "1⛽️🚦".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);

    // Bob registers a subdomain of length 3
    let ix = register(
        register::Accounts {
            name_auctioning_program: &register::NAME_AUCTIONING,
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
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.total_sub_created = 3;
    assert_eq!(registry, expected_registry);

    // Verify fees received
    let acc = prg_test_ctx
        .banks_client
        .get_account(*alice_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    assert_eq!(
        token_account.amount,
        8_000_000 + 10_000_000 + 8_000_000 + 7_000_000
    );

    // Unregister all subs
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![
            unregister(
                unregister::Accounts {
                    system_program: &system_program::ID,
                    spl_name_service: &spl_name_service::ID,
                    registry: &registry_key,
                    sub_domain_account: &get_subdomain_key("1".to_owned(), &name_key),
                    domain_owner: &bob.pubkey(),
                },
                unregister::Params {},
            ),
            unregister(
                unregister::Accounts {
                    system_program: &system_program::ID,
                    spl_name_service: &spl_name_service::ID,
                    registry: &registry_key,
                    sub_domain_account: &get_subdomain_key("1⛽️".to_owned(), &name_key),
                    domain_owner: &bob.pubkey(),
                },
                unregister::Params {},
            ),
            unregister(
                unregister::Accounts {
                    system_program: &system_program::ID,
                    spl_name_service: &spl_name_service::ID,
                    registry: &registry_key,
                    sub_domain_account: &get_subdomain_key("1⛽️🚦".to_owned(), &name_key),
                    domain_owner: &bob.pubkey(),
                },
                unregister::Params {},
            ),
        ],
        vec![&bob],
    )
    .await
    .unwrap();
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registry: Registry = Registry::deserialize(&mut &acc.data[..]).unwrap();
    expected_registry.total_sub_created = 0;
    assert_eq!(registry, expected_registry);

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
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
}