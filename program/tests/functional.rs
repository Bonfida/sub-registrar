use solana_program::program_pack::Pack;
use sub_register::{
    entrypoint::process_instruction,
    instruction::{
        admin_register, admin_revoke, close_registrar, create_registrar, delete_subrecord,
        edit_registrar, register, unregister,
    },
    state::{
        nft_mint_record::NftMintRecord, registry::Registrar, schedule::Price, subrecord::SubRecord,
        FEE_ACC_OWNER, NAME_AUCTIONING,
    },
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
async fn test_functional() {
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
    let (registry_key, _) = Registrar::find_key(&name_key, &alice.pubkey(), &sub_register::ID);
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
            nft_gated_collection: None,
            max_nft_mint: 0,
            allow_revoke: false,
            authority: alice.pubkey(),
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
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    // Increase vec size
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: Some(1),
            new_collection: None,
            new_authority: None,
            new_mint: None,
            new_fee_account: None,
            disable_nft_gate: false,
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

    // Decrease vec size
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_collection: None,
            new_authority: None,
            new_max_nft_mint: None,
            new_mint: None,
            disable_nft_gate: false,
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
            ]),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    let sub_domain = "some-test".to_string();
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
    let ix = unregister(
        unregister::Accounts {
            system_program: &system_program::ID,
            spl_name_service: &spl_name_service::ID,
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            domain_owner: &bob.pubkey(),
            sub_record: &subrecord_key,
        },
        unregister::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    let sub_domain = "some-test".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let sub_domain_key_to_unreg_1 = sub_domain_key.clone();
    let (subrecord_key_to_unreg_1, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);
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
            nft_account: None,
            nft_metadata_account: None,
            nft_mint_record: None,
            sub_record: &subrecord_key_to_unreg_1.clone(),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    // Admin register
    let sub_domain = "some-admin-test".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let sub_domain_key_to_unreg_2 = sub_domain_key.clone();
    let (subrecord_key_to_unreg_2, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);
    let ix = admin_register(
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
            sub_record: &subrecord_key_to_unreg_2.clone(),
        },
        admin_register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    // Unregister all domains
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![unregister(
            unregister::Accounts {
                system_program: &system_program::ID,
                spl_name_service: &spl_name_service::ID,
                registrar: &registry_key,
                sub_domain_account: &sub_domain_key_to_unreg_2,
                domain_owner: &alice.pubkey(),
                sub_record: &subrecord_key_to_unreg_2,
            },
            unregister::Params {},
        )],
        vec![&alice],
    )
    .await
    .unwrap();
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![unregister(
            unregister::Accounts {
                system_program: &system_program::ID,
                spl_name_service: &spl_name_service::ID,
                registrar: &registry_key,
                sub_domain_account: &sub_domain_key_to_unreg_1,
                domain_owner: &bob.pubkey(),
                sub_record: &subrecord_key_to_unreg_1,
            },
            unregister::Params {},
        )],
        vec![&bob],
    )
    .await
    .unwrap();

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
            allow_revoke: true,
            nft_gated_collection: Some(common::metadata::COLLECTION_KEY),
            max_nft_mint: 1,
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

    // Test: register with NFT
    let sub_domain = "some-test-22345".to_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(sub_domain.clone(), &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(sub_domain.clone(), &name_key);
    let (subrecord_key, _) = SubRecord::find_key(&sub_domain_key, &sub_register::ID);
    let (nft_mint_record, _) =
        NftMintRecord::find_key(&common::metadata::NFT_MINT, &sub_register::ID);
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
            nft_mint_record: Some(&nft_mint_record),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    // Test: edit registrar to remove NFT collection & register without NFT
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            disable_nft_gate: true,
            new_collection: None,
            new_authority: Some(alice.pubkey()),
            new_mint: None,
            new_fee_account: None,
            new_price_schedule: None,
            new_max_nft_mint: None,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    let sub_domain = "some-test-1343123422345".to_string();
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
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&nft_mint_record),
        },
        register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    // Delete domain via SNS
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
            sub_record: &SubRecord::find_key(&sub_domain_key, &sub_register::ID).0,
        },
        delete_subrecord::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    // Test: revoke sub
    let sub_domain = "some-test-1343123422ghjk345".to_string();
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
            bonfida_fee_account,
            nft_account: Some(&bob_nft_account),
            nft_metadata_account: Some(&common::metadata::NFT_METADATA_KEY),
            sub_record: &subrecord_key,
            nft_mint_record: Some(&nft_mint_record),
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
            authority: &alice.pubkey(),
            spl_name_service: &spl_name_service::ID,
            sub_record: &subrecord_key,
            name_class: &Pubkey::default(),
            sub_owner: &bob.pubkey(),
            parent_domain: &name_key,
        },
        admin_revoke::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
}
