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
async fn test_functional() {
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
            nft_gated_collection: None,
            max_nft_mint: 0,
            allow_revoke: false,
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

    // Increase vec size
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_max_nft_mint: Some(1),
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

    // Decrease vec size
    let var_name = vec![
        Price {
            length: 1,
            price: 10_000_000,
        },
        Price {
            length: 2,
            price: 10_000_000,
        },
    ];
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
            new_authority: None,
            new_max_nft_mint: None,
            new_mint: None,
            new_fee_account: None,
            new_price_schedule: Some(var_name),
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

    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let sub_domain_key_to_unreg_1 = sub_domain_key;
    let (subrecord_key_to_unreg_1, _) =
        SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
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
    let sub_domain = random_string();
    let sub_domain_key = sub_register::utils::get_subdomain_key(&sub_domain, &name_key);
    let sub_reverse_key = sub_register::utils::get_subdomain_reverse(&sub_domain, &name_key);
    let sub_domain_key_to_unreg_2 = sub_domain_key;
    let (subrecord_key_to_unreg_2, _) =
        SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID);
    let ix = admin_register(
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
                mint_record: None,
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
                mint_record: None,
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
            max_nft_mint: 2,
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

    // Test: register with NFT
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

    // Test: edit registrar to remove NFT collection & register without NFT
    let ix = edit_registrar(
        edit_registrar::Accounts {
            system_program: &system_program::ID,
            authority: &alice.pubkey(),
            registrar: &registry_key,
        },
        edit_registrar::Params {
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
    let ix = delete_subdomain_record(
        delete_subdomain_record::Accounts {
            sub_domain: &sub_domain_key,
            lamports_target: &bob.pubkey(),
            sub_record: &SubDomainRecord::find_key(&sub_domain_key, &sub_register::ID).0,
            mint_record: Some(&mint_record),
            registrar: &registry_key,
        },
        delete_subdomain_record::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    // Test: revoke sub
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
            mint_record: Some(&mint_record),
        },
        admin_revoke::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    // Test: revoke sub via NFT owner
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
    // Transfer domain to Alice
    let ix = spl_name_service::instruction::transfer(
        spl_name_service::ID,
        alice.pubkey(),
        sub_domain_key,
        bob.pubkey(),
        None,
    )
    .unwrap();
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let ix = nft_owner_revoke(
        nft_owner_revoke::Accounts {
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            nft_owner: &bob.pubkey(),
            nft_account: &bob_nft_account,
            spl_name_service: &spl_name_service::ID,
            nft_metadata: &common::metadata::NFT_METADATA_KEY,
            sub_record: &subrecord_key,
            name_class: &Pubkey::default(),
            sub_owner: &alice.pubkey(),
            parent_domain: &name_key,
            nft_mint_record: &mint_record,
        },
        nft_owner_revoke::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
}
