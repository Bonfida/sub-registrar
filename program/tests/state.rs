//! Tests of state integrity

use solana_program::program_pack::Pack;
use sub_register::{
    entrypoint::process_instruction,
    instruction::{
        admin_register, admin_revoke, close_registrar, create_registrar, delete_subrecord,
        edit_registrar, nft_owner_revoke, register, unregister,
    },
    state::{
        mint_record::MintRecord, registry::Registrar, schedule::Price, subrecord::SubRecord, Tag,
        FEE_ACC_OWNER, NAME_AUCTIONING,
    },
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
    program_test.add_program("name_auctioning", NAME_AUCTIONING, None);
    program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);

    // Add mock NFT & collection
    let mut data: Vec<u8> = vec![];
    common::metadata::get_metadata()
        .serialize(&mut data)
        .unwrap();
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
    common::metadata::get_nft_account(&bob.pubkey()).pack_into_slice(&mut data);
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

    // Alice creates registry
    let (registry_key, nonce) = Registrar::find_key(&name_key, &alice.pubkey(), &sub_register::ID);
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
            max_nft_mint: 0,
            allow_revoke: false,
            nft_gated_collection: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    let mut expected_registrar = Registrar {
        max_nft_mint: 0,
        allow_revoke: false,
        nft_gated_collection: None,
        tag: Tag::Registrar,
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
    assert_eq!(registrar, expected_registrar);

    // Increase vec size
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.price_schedule = vec![
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
    assert_eq!(registrar, expected_registrar);

    // Decrease vec size
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.price_schedule = vec![
        Price {
            length: 1,
            price: 10_000_000,
        },
        Price {
            length: 2,
            price: 8_000_000,
        },
    ];
    assert_eq!(registrar, expected_registrar);

    // Change mint
    let new_mint = Pubkey::new_unique();
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.mint = new_mint;
    assert_eq!(registrar, expected_registrar);

    // Change mint back so we don't have to create a new token
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.mint = mint;
    assert_eq!(registrar, expected_registrar);

    // Change fee account
    let new_fee_account = Pubkey::new_unique();
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.fee_account = new_fee_account;
    assert_eq!(registrar, expected_registrar);
    // Change it back
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.fee_account = *alice_fee_account;
    assert_eq!(registrar, expected_registrar);

    // Change authority
    let new_authority = Keypair::new();
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: Some(5),
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.authority = new_authority.pubkey();
    expected_registrar.max_nft_mint = 5;
    assert_eq!(registrar, expected_registrar);

    // Change authority back to alice
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &new_authority.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.authority = alice.pubkey();
    assert_eq!(registrar, expected_registrar);

    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);

    // Bob registers a subdomain
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
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account: &bonfida_fee_account,
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

    // Verify state
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.total_sub_created = 1;
    assert_eq!(registrar, expected_registrar);

    let ix = unregister(
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.total_sub_created = 0;
    assert_eq!(registrar, expected_registrar);

    // Verify fees received
    let acc = prg_test_ctx
        .banks_client
        .get_account(*alice_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    let mut total_fees = (8_000_000 * 5) / 100;
    assert_eq!(token_account.amount, 8_000_000 - total_fees);

    // Change price schedule + register + verify
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.price_schedule = vec![
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
    assert_eq!(registrar, expected_registrar);

    let sub_domain = "1".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);

    // Bob registers a subdomain of length 1
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
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account: &bonfida_fee_account,
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
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.total_sub_created = 1;
    assert_eq!(registrar, expected_registrar);

    // Verify fees received
    let acc = prg_test_ctx
        .banks_client
        .get_account(*alice_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    total_fees += (10_000_000 * 5) / 100;
    assert_eq!(token_account.amount, 8_000_000 + 10_000_000 - total_fees);

    let sub_domain = "1⛽️".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);

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
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account: &bonfida_fee_account,
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
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.total_sub_created = 2;
    assert_eq!(registrar, expected_registrar);
    let acc = prg_test_ctx
        .banks_client
        .get_account(subrecord_key)
        .await
        .unwrap()
        .unwrap();
    let subrecord: SubRecord = SubRecord::deserialize(&mut &acc.data[..]).unwrap();
    assert_eq!(subrecord, SubRecord::new(registry_key));

    // Verify fees received
    let acc = prg_test_ctx
        .banks_client
        .get_account(*alice_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    total_fees += (8_000_000 * 5) / 100;
    assert_eq!(
        token_account.amount,
        8_000_000 + 10_000_000 + 8_000_000 - total_fees
    );

    let sub_domain = "1⛽️🚦".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);

    // Bob registers a subdomain of length 3
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
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account: &bonfida_fee_account,
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
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.total_sub_created = 3;
    assert_eq!(registrar, expected_registrar);
    let acc = prg_test_ctx
        .banks_client
        .get_account(subrecord_key)
        .await
        .unwrap()
        .unwrap();
    let subrecord: SubRecord = SubRecord::deserialize(&mut &acc.data[..]).unwrap();
    assert_eq!(subrecord, SubRecord::new(registry_key));

    // Verify fees received
    let acc = prg_test_ctx
        .banks_client
        .get_account(*alice_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    total_fees += (7_000_000 * 5) / 100;
    assert_eq!(
        token_account.amount,
        8_000_000 + 10_000_000 + 8_000_000 + 7_000_000 - total_fees
    );

    // Unregister all subs
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![
            unregister(
                unregister::Accounts {
                    mint_record: None,
                    system_program: &system_program::ID,
                    spl_name_service: &spl_name_service::ID,
                    registrar: &registry_key,
                    sub_domain_account: &get_subdomain_key("1".to_owned(), &name_key),
                    domain_owner: &bob.pubkey(),
                    sub_record: &SubRecord::find_key(
                        &get_subdomain_key("1".to_owned(), &name_key),
                        &sub_register::ID,
                    )
                    .0,
                },
                unregister::Params {},
            ),
            unregister(
                unregister::Accounts {
                    mint_record: None,
                    system_program: &system_program::ID,
                    spl_name_service: &spl_name_service::ID,
                    registrar: &registry_key,
                    sub_domain_account: &get_subdomain_key("1⛽️".to_owned(), &name_key),
                    domain_owner: &bob.pubkey(),
                    sub_record: &SubRecord::find_key(
                        &get_subdomain_key("1⛽️".to_owned(), &name_key),
                        &sub_register::ID,
                    )
                    .0,
                },
                unregister::Params {},
            ),
            unregister(
                unregister::Accounts {
                    mint_record: None,
                    system_program: &system_program::ID,
                    spl_name_service: &spl_name_service::ID,
                    registrar: &registry_key,
                    sub_domain_account: &get_subdomain_key("1⛽️🚦".to_owned(), &name_key),
                    domain_owner: &bob.pubkey(),
                    sub_record: &SubRecord::find_key(
                        &get_subdomain_key("1⛽️🚦".to_owned(), &name_key),
                        &sub_register::ID,
                    )
                    .0,
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.total_sub_created = 0;
    assert_eq!(registrar, expected_registrar);

    // Admin register
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);
    sign_send_instructions(
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
                registrar: &registry_key,
                parent_domain_account: &name_key,
                sub_domain_account: &sub_domain_key,
                sub_reverse_account: &sub_reverse_key,
                authority: &alice.pubkey(),
                sub_record: &subrecord_key,
            },
            admin_register::Params {
                domain: format!("\0{}", sub_domain),
            },
        )],
        vec![&alice],
    )
    .await
    .unwrap();
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.total_sub_created = 1;
    assert_eq!(registrar, expected_registrar);

    // Check subrecord
    let acc = prg_test_ctx
        .banks_client
        .get_account(subrecord_key)
        .await
        .unwrap()
        .unwrap();
    let subrecord: SubRecord = SubRecord::deserialize(&mut &acc.data[..]).unwrap();
    assert_eq!(subrecord, SubRecord::new(registry_key));

    // Unregister admin created sub
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![unregister(
            unregister::Accounts {
                mint_record: None,
                system_program: &system_program::ID,
                spl_name_service: &spl_name_service::ID,
                registrar: &registry_key,
                sub_domain_account: &sub_domain_key,
                domain_owner: &alice.pubkey(),
                sub_record: &subrecord_key,
            },
            unregister::Params {},
        )],
        vec![&alice],
    )
    .await
    .unwrap();
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.total_sub_created = 0;
    assert_eq!(registrar, expected_registrar);
    let acc = prg_test_ctx
        .banks_client
        .get_account(subrecord_key)
        .await
        .unwrap();
    assert!(acc.is_none());

    // Close registry
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

    // Check fee account
    let acc = prg_test_ctx
        .banks_client
        .get_account(*bonfida_fee_account)
        .await
        .unwrap()
        .unwrap();
    let token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(&acc.data[..]).unwrap();
    assert_eq!(token_account.amount, total_fees);

    ////////////////////////////////////////
    //
    // Test with NFT gated registrar
    //
    ////////////////////////////////////////

    // Test: create a registrar with nft collection
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
            nft_gated_collection: Some(common::metadata::COLLECTION_KEY),
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
    let mut expected_registrar = Registrar {
        max_nft_mint: 4,
        allow_revoke: true,
        nft_gated_collection: Some(common::metadata::COLLECTION_KEY),
        tag: Tag::Registrar,
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
    let acc = prg_test_ctx
        .banks_client
        .get_account(registry_key)
        .await
        .unwrap()
        .unwrap();
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    assert_eq!(expected_registrar, registrar);

    // Test: edit the registrar
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: Some(5),
            new_authority: Some(alice.pubkey()),
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    expected_registrar.max_nft_mint = 5;
    expected_registrar.price_schedule = vec![
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
    assert_eq!(registrar, expected_registrar);

    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: None,
            new_authority: Some(alice.pubkey()),
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
    let registrar: Registrar = Registrar::deserialize(&mut &acc.data[..]).unwrap();
    assert_eq!(registrar, expected_registrar);

    ////////////////////////////////////////
    //
    // Test how revoke instruction affect the state
    //
    ////////////////////////////////////////

    // First, create sub
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_to_revoke = sub_domain_key.clone();
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record_key, _) = MintRecord::find_key(&common::metadata::NFT_MINT, &sub_register::ID);
    // Bob registers a subdomain
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
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record_key),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    // Verify state
    let mint_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<MintRecord>(mint_record_key)
        .await
        .unwrap();
    let mut expected_mint_record = MintRecord {
        tag: Tag::MintRecord,
        count: 1,
    };
    assert_eq!(mint_record, expected_mint_record);
    let sub_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<SubRecord>(subrecord_key)
        .await
        .unwrap();
    let expected_sub_record = SubRecord {
        tag: Tag::SubRecord,
        registrar: registry_key,
        mint_record: Some(mint_record_key),
    };
    assert_eq!(sub_record, expected_sub_record);

    // Creates another sub
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record_key, _) = MintRecord::find_key(&common::metadata::NFT_MINT, &sub_register::ID);
    // Bob registers a subdomain
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
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record_key),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    // Verify state
    let mint_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<MintRecord>(mint_record_key)
        .await
        .unwrap();
    expected_mint_record.count += 1;
    assert_eq!(mint_record, expected_mint_record);
    let registrar = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<Registrar>(registry_key)
        .await
        .unwrap();
    expected_registrar.total_sub_created = 2;
    assert_eq!(registrar, expected_registrar);

    // Admin revoke
    let ix = admin_revoke(
        admin_revoke::Accounts {
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            authority: &alice.pubkey(),
            spl_name_service: &spl_name_service::ID,
            sub_record: &subrecord_key,
            name_class: &Pubkey::default(),
            sub_owner: &bob.pubkey(),
            parent_domain: &name_key,
            mint_record: Some(&mint_record_key),
        },
        admin_revoke::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    // Verify state
    let mint_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<MintRecord>(mint_record_key)
        .await
        .unwrap();
    expected_mint_record.count -= 1;
    assert_eq!(mint_record, expected_mint_record);
    let registrar = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<Registrar>(registry_key)
        .await
        .unwrap();
    expected_registrar.total_sub_created -= 1;
    assert_eq!(registrar, expected_registrar);

    // Transfer domain then NFT revoke
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
    let ix = nft_owner_revoke(
        nft_owner_revoke::Accounts {
            registrar: &&registry_key,
            sub_domain_account: &sub_to_revoke,
            sub_record: &SubRecord::find_key(&sub_to_revoke, &sub_register::ID).0,
            sub_owner: &bob.pubkey(),
            parent_domain: &name_key,
            nft_account: &bob_nft_account,
            nft_metadata: &common::metadata::NFT_METADATA_KEY,
            nft_owner: &bob.pubkey(),
            name_class: &Pubkey::default(),
            nft_mint_record: &mint_record_key,
            spl_name_service: &spl_name_service::ID,
        },
        nft_owner_revoke::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    // Verify state
    let mint_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<MintRecord>(mint_record_key)
        .await
        .unwrap();
    expected_mint_record.count -= 1;
    assert_eq!(mint_record, expected_mint_record);
    let registrar = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<Registrar>(registry_key)
        .await
        .unwrap();
    expected_registrar.total_sub_created -= 1;
    assert_eq!(registrar, expected_registrar);

    ////////////////////////////////////////
    //
    // Test how register/unregister affect MintRecord
    //
    ////////////////////////////////////////

    // Creates another sub
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record_key, _) = MintRecord::find_key(&common::metadata::NFT_MINT, &sub_register::ID);
    // Bob registers a subdomain
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
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record_key),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    // Verify state
    let mint_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<MintRecord>(mint_record_key)
        .await
        .unwrap();
    expected_mint_record.count += 1;
    assert_eq!(mint_record, expected_mint_record);
    let registrar = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<Registrar>(registry_key)
        .await
        .unwrap();
    expected_registrar.total_sub_created += 1;
    assert_eq!(registrar, expected_registrar);

    // Unregister
    let ix = unregister(
        unregister::Accounts {
            system_program: &system_program::ID,
            spl_name_service: &spl_name_service::ID,
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            domain_owner: &bob.pubkey(),
            sub_record: &subrecord_key,
            mint_record: Some(&mint_record_key),
        },
        unregister::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    // Verify state
    let mint_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<MintRecord>(mint_record_key)
        .await
        .unwrap();
    expected_mint_record.count -= 1;
    assert_eq!(mint_record, expected_mint_record);
    let registrar = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<Registrar>(registry_key)
        .await
        .unwrap();
    expected_registrar.total_sub_created -= 1;
    assert_eq!(registrar, expected_registrar);

    // Creates another sub
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (mint_record_key, _) = MintRecord::find_key(&common::metadata::NFT_MINT, &sub_register::ID);
    // Bob registers a subdomain
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
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &bob.pubkey(),
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&mint_record_key),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    // Verify state
    let mint_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<MintRecord>(mint_record_key)
        .await
        .unwrap();
    expected_mint_record.count += 1;
    assert_eq!(mint_record, expected_mint_record);
    let registrar = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<Registrar>(registry_key)
        .await
        .unwrap();
    expected_registrar.total_sub_created += 1;
    assert_eq!(registrar, expected_registrar);

    // Delete via SPL name service
    let ix = spl_name_service::instruction::delete(
        spl_name_service::ID,
        sub_domain_key,
        bob.pubkey(),
        bob.pubkey(),
    )
    .unwrap();
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let ix = delete_subrecord(
        delete_subrecord::Accounts {
            sub_domain: &sub_domain_key,
            lamports_target: &bob.pubkey(),
            sub_record: &subrecord_key,
            mint_record: Some(&mint_record_key),
            registrar: &registry_key,
        },
        delete_subrecord::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    // Verify state
    let mint_record = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<MintRecord>(mint_record_key)
        .await
        .unwrap();
    expected_mint_record.count -= 1;
    assert_eq!(mint_record, expected_mint_record);
    let registrar = prg_test_ctx
        .banks_client
        .get_account_data_with_borsh::<Registrar>(registry_key)
        .await
        .unwrap();
    expected_registrar.total_sub_created -= 1;
    assert_eq!(registrar, expected_registrar);

    // Close registrar
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
}
