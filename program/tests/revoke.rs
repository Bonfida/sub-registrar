use solana_program::{instruction::InstructionError, program_pack::Pack};
use solana_program_test::BanksClientError;
use solana_sdk::transaction::TransactionError;
use sub_register::{
    entrypoint::process_instruction,
    error::SubRegisterError,
    instruction::{admin_register, admin_revoke, create_registrar, register, unregister},
    state::{
        registry::Registrar,
        schedule::Price,
        subdomain_record::{SubDomainRecord, REVOKE_EXPIRY_DELAY_SECONDS_MIN},
        FEE_ACC_OWNER, ROOT_DOMAIN_ACCOUNT,
    },
};

use crate::common::utils::ProgramTestContextExtended;
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
async fn test_revoke_impersonation_safety() {
    // Create program and test environment
    use common::utils::{random_string, sign_send_instructions};

    pub const NUMBER_OF_ACTORS: usize = 3;
    // Owns the .sol, creates and administers the registry
    pub const ALICE: usize = 0;
    pub const BOB: usize = 1;
    pub const CHARLIE: usize = 2;

    let keypairs = (0..NUMBER_OF_ACTORS)
        .map(|n| Keypair::new())
        .collect::<Vec<_>>();

    println!("[+] Alice key {}", keypairs[ALICE].pubkey());
    println!("[+] Bob key {}", keypairs[BOB].pubkey());
    println!("[+] Charlie key {}", keypairs[CHARLIE].pubkey());

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

    // Create and fund actor accounts
    for k in &keypairs {
        program_test.add_account(
            k.pubkey(),
            Account {
                lamports: 100_000_000_000,
                ..Account::default()
            },
        );
    }

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
        owner: keypairs[ALICE].pubkey(),
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
        common::utils::mint_bootstrap(None, 6, &mut program_test, &keypairs[ALICE].pubkey());

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    // Create ATAs
    let instructions = keypairs
        .iter()
        .map(|k| {
            create_associated_token_account(
                &prg_test_ctx.payer.pubkey(),
                &k.pubkey(),
                &mint,
                &spl_token::ID,
            )
        })
        .collect();
    sign_send_instructions(&mut prg_test_ctx, instructions, vec![])
        .await
        .unwrap();

    let atas = keypairs
        .iter()
        .map(|k| get_associated_token_address(&k.pubkey(), &mint))
        .collect::<Vec<_>>();

    sign_send_instructions(
        &mut prg_test_ctx,
        atas.iter()
            .map(|a| {
                spl_token::instruction::mint_to(
                    &spl_token::ID,
                    &mint,
                    a,
                    &keypairs[ALICE].pubkey(),
                    &[],
                    10_000_000_000,
                )
                .unwrap()
            })
            .collect(),
        vec![&keypairs[ALICE]],
    )
    .await
    .unwrap();

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
            domain_owner: &keypairs[ALICE].pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_name_program_id: &spl_name_service::ID,
        },
        create_registrar::Params {
            mint,
            fee_account: atas[ALICE],
            nft_gated_collection: None,
            max_nft_mint: 0,
            allow_revoke: true,
            authority: keypairs[ALICE].pubkey(),
            price_schedule: common::utils::serialize_price_schedule(&[
                Price {
                    length: 1,
                    price: 10_000_000,
                },
                Price {
                    length: 2,
                    price: 10_000_000,
                },
            ]),
            revoke_expiry_delay: REVOKE_EXPIRY_DELAY_SECONDS_MIN,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[ALICE]])
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
            fee_account: &atas[ALICE],
            fee_source: &atas[BOB],
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &keypairs[BOB].pubkey(),
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
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[BOB]])
        .await
        .unwrap();

    // Bob unregisters successfully
    let ix = unregister(
        unregister::Accounts {
            system_program: &system_program::ID,
            spl_name_service: &spl_name_service::ID,
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            domain_owner: &keypairs[BOB].pubkey(),
            sub_record: &subrecord_key,
            mint_record: None,
        },
        unregister::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[BOB]])
        .await
        .unwrap();

    // Alice registers that same subdomain successfully
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: &atas[ALICE],
            fee_source: &atas[ALICE],
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &keypairs[ALICE].pubkey(),
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
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[ALICE]])
        .await
        .unwrap();

    // Alice unregisters the domain successfully
    let ix = unregister(
        unregister::Accounts {
            system_program: &system_program::ID,
            spl_name_service: &spl_name_service::ID,
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            domain_owner: &keypairs[ALICE].pubkey(),
            sub_record: &subrecord_key,
            mint_record: None,
        },
        unregister::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[ALICE]])
        .await
        .unwrap();

    // To avoid a no-op
    prg_test_ctx
        .warp_forward_force_reward_interval_end()
        .unwrap();

    // Bob registers the subdomain again
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: &atas[ALICE],
            fee_source: &atas[BOB],
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &keypairs[BOB].pubkey(),
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
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[BOB]])
        .await
        .unwrap();

    // This time, Alice revokes it
    let ix = admin_revoke(
        admin_revoke::Accounts {
            registrar: &registry_key,
            sub_domain_account: &sub_domain_key,
            authority: &keypairs[ALICE].pubkey(),
            spl_name_service: &spl_name_service::ID,
            sub_record: &subrecord_key,
            name_class: &Pubkey::default(),
            sub_owner: &keypairs[BOB].pubkey(),
            parent_domain: &name_key,
            mint_record: None,
        },
        admin_revoke::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[ALICE]])
        .await
        .unwrap();

    // Alice attempts to register the subdomain after, but fails
    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: &atas[ALICE],
            fee_source: &atas[ALICE],
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &keypairs[ALICE].pubkey(),
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
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[ALICE]]).await;
    if let Err(BanksClientError::TransactionError(TransactionError::InstructionError(
        0,
        InstructionError::Custom(n),
    ))) = res
    {
        assert_eq!(n, SubRegisterError::RevokedSubdomainNotExpired as u32)
    }

    // Alice attempts to admin register the subdomain after, but fails
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
            sub_record: &subrecord_key,
            authority: &keypairs[ALICE].pubkey(),
        },
        admin_register::Params {
            domain: format!("\0{}", sub_domain),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[ALICE]]).await;
    if let Err(BanksClientError::TransactionError(TransactionError::InstructionError(
        0,
        InstructionError::Custom(n),
    ))) = res
    {
        assert_eq!(n, SubRegisterError::RevokedSubdomainNotExpired as u32)
    }

    // We then wait for a week and retry successfully
    prg_test_ctx
        .warp_forward(REVOKE_EXPIRY_DELAY_SECONDS_MIN)
        .await
        .unwrap();

    let ix = register(
        register::Accounts {
            sns_registrar_program: &sns_registrar::ID,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
            spl_name_service: &spl_name_service::ID,
            rent_sysvar: &sysvar::rent::id(),
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup_class: &sns_registrar::central_state::KEY,
            fee_account: &atas[ALICE],
            fee_source: &atas[ALICE],
            registrar: &registry_key,
            parent_domain_account: &name_key,
            sub_domain_account: &sub_domain_key,
            sub_reverse_account: &sub_reverse_key,
            fee_payer: &keypairs[ALICE].pubkey(),
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
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&keypairs[ALICE]])
        .await
        .unwrap();
}
