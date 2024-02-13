use std::convert::TryFrom;

use mpl_token_metadata::accounts::Metadata;
use solana_program::{program_error::ProgramError, program_pack::Pack};

use crate::{error::SubRegisterError, state::schedule::Schedule};

use {
    solana_program::{account_info::AccountInfo, hash::hashv, pubkey::Pubkey},
    spl_name_service::state::{get_seeds_and_key, HASH_PREFIX},
    unicode_segmentation::UnicodeSegmentation,
};

pub fn get_domain_price(domain: String, schedule: &Schedule) -> u64 {
    let ui_domain = domain.strip_prefix('\0').unwrap();
    let len = ui_domain.graphemes(true).count() as u64;
    for price in schedule {
        if len == price.length {
            return price.price;
        }
    }
    // Less expensive price
    let last = schedule.last().unwrap();
    last.price
}

pub fn get_subdomain_key(ui_subdomain: &str, parent: &Pubkey) -> Pubkey {
    let domain = format!("\0{ui_subdomain}");
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &domain).as_bytes()])
        .as_ref()
        .to_vec();

    let (name_account_key, _) =
        get_seeds_and_key(&spl_name_service::ID, hashed_name, None, Some(parent));
    name_account_key
}

pub fn get_subdomain_reverse(ui_subdomain: &str, parent: &Pubkey) -> Pubkey {
    let subdomain_key = get_subdomain_key(ui_subdomain, parent);
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &subdomain_key.to_string()).as_bytes()])
        .as_ref()
        .to_vec();

    let (name_account_key, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed_name,
        Some(&sns_registrar::central_state::KEY),
        Some(parent),
    );
    name_account_key
}

// Assumes the account is owned by SPL Token !!!!
pub fn check_nft_holding_and_get_mint(
    nft_account: &AccountInfo,
    expected_owner: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    // Deserialize token account
    let token_acc = spl_token::state::Account::unpack(&nft_account.data.borrow())?;

    // Check correct owner
    if token_acc.owner != *expected_owner {
        return Err(SubRegisterError::WrongOwner.into());
    }
    // Check correct amount
    if token_acc.amount != 1 {
        return Err(SubRegisterError::MustHoldOneNFt.into());
    }

    Ok(token_acc.mint)
}

// Assumes the account is owned by MPL token metadata !!!!
pub fn check_metadata(
    nft_metadata_account: &AccountInfo,
    expected_collection: &Pubkey,
) -> Result<(), ProgramError> {
    // Deserialize metadata

    let metadata = Metadata::try_from(nft_metadata_account)?;

    if let Some(collection) = metadata.collection {
        // Check collection is verified
        if !collection.verified {
            return Err(SubRegisterError::InvalidCollection.into());
        }
        // Check collection
        if collection.key != *expected_collection {
            return Err(SubRegisterError::InvalidCollection.into());
        }

        return Ok(());
    }

    Err(SubRegisterError::MustHaveCollection.into())
}

#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;
    use mpl_token_metadata::types::{Collection, Key};

    use super::*;
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn test_price_logic() {
        use crate::state::schedule::Price;
        let schedule: Schedule = vec![
            Price {
                length: 1,
                price: 100,
            },
            Price {
                length: 2,
                price: 90,
            },
            Price {
                length: 3,
                price: 80,
            },
            Price {
                length: 4,
                price: 70,
            },
            Price {
                length: 5,
                price: 60,
            },
        ];

        assert_eq!(get_domain_price("\x001".to_string(), &schedule), 100);
        assert_eq!(get_domain_price("\x0011".to_string(), &schedule), 90);
        assert_eq!(get_domain_price("\x00111".to_string(), &schedule), 80);
        assert_eq!(get_domain_price("\x001111".to_string(), &schedule), 70);
        assert_eq!(get_domain_price("\x0011111".to_string(), &schedule), 60);
        assert_eq!(get_domain_price("\x00111111".to_string(), &schedule), 60);
        assert_eq!(
            get_domain_price("\x001111111111".to_string(), &schedule),
            60
        );

        assert_eq!(get_domain_price("\x00😀".to_string(), &schedule), 100);
        assert_eq!(get_domain_price("\x001😀".to_string(), &schedule), 90);
        assert_eq!(get_domain_price("\x001😀1".to_string(), &schedule), 80);
        assert_eq!(get_domain_price("\x0011😀1".to_string(), &schedule), 70);
        assert_eq!(get_domain_price("\x00111😀1".to_string(), &schedule), 60);
        assert_eq!(get_domain_price("\x0011😀111".to_string(), &schedule), 60);
        assert_eq!(
            get_domain_price("\x0011😀1111111".to_string(), &schedule),
            60
        );
    }

    #[test]
    fn test_check_nft_holding_and_get_mint() {
        let owner = Pubkey::new_unique();

        let mut data = spl_token::state::Account {
            mint: Pubkey::new_unique(),
            owner,
            amount: 1,
            state: spl_token::state::AccountState::Initialized,
            ..spl_token::state::Account::default()
        };
        let mut buf: Vec<u8> = vec![0; spl_token::state::Account::LEN];
        spl_token::state::Account::pack(data, &mut buf).unwrap();

        // Correct owner with 1 token
        check_nft_holding_and_get_mint(
            &AccountInfo {
                key: &Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
                owner: &spl_token::ID,
                lamports: Rc::new(RefCell::new(&mut 0)),
                data: Rc::new(RefCell::new(&mut buf[..])),
                executable: false,
                rent_epoch: 0,
            },
            &owner,
        )
        .unwrap();

        // Wrong owner with 1 token
        let res = check_nft_holding_and_get_mint(
            &AccountInfo {
                key: &Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
                owner: &spl_token::ID,
                lamports: Rc::new(RefCell::new(&mut 0)),
                data: Rc::new(RefCell::new(&mut buf[..])),
                executable: false,
                rent_epoch: 0,
            },
            &Pubkey::new_unique(),
        );
        assert!(res.is_err());

        // Correct owner with 0 token
        data.amount = 0;
        let mut buf: Vec<u8> = vec![0; spl_token::state::Account::LEN];
        spl_token::state::Account::pack(data, &mut buf).unwrap();
        let res = check_nft_holding_and_get_mint(
            &AccountInfo {
                key: &Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
                owner: &spl_token::ID,
                lamports: Rc::new(RefCell::new(&mut 0)),
                data: Rc::new(RefCell::new(&mut buf[..])),
                executable: false,
                rent_epoch: 0,
            },
            &owner,
        );
        assert!(res.is_err());

        // Wrong owner with 0 token
        let res = check_nft_holding_and_get_mint(
            &AccountInfo {
                key: &Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
                owner: &spl_token::ID,
                lamports: Rc::new(RefCell::new(&mut 0)),
                data: Rc::new(RefCell::new(&mut buf[..])),
                executable: false,
                rent_epoch: 0,
            },
            &Pubkey::new_unique(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_check_metadata() {
        let collection = Pubkey::new_unique();
        let metadata = Metadata {
            programmable_config: None,
            key: Key::MetadataV1,
            update_authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            name: "".to_string(),
            symbol: "".to_string(),
            uri: "".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            primary_sale_happened: true,
            is_mutable: true,
            edition_nonce: Some(255),
            token_standard: None,
            collection: Some(Collection {
                verified: true,
                key: collection,
            }),
            uses: None,
            collection_details: None,
        };
        let mut buf = vec![];
        metadata.serialize(&mut buf).unwrap();
        check_metadata(
            &AccountInfo {
                key: &Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
                owner: &mpl_token_metadata::ID,
                lamports: Rc::new(RefCell::new(&mut 0)),
                data: Rc::new(RefCell::new(&mut buf[..])),
                executable: false,
                rent_epoch: 0,
            },
            &collection,
        )
        .unwrap();

        // Unverified collection
        let metadata = Metadata {
            programmable_config: None,
            key: Key::MetadataV1,
            update_authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            name: "".to_string(),
            symbol: "".to_string(),
            uri: "".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            primary_sale_happened: true,
            is_mutable: true,
            edition_nonce: Some(255),
            token_standard: None,
            collection: Some(Collection {
                verified: false,
                key: collection,
            }),
            uses: None,
            collection_details: None,
        };
        let mut buf = vec![];
        metadata.serialize(&mut buf).unwrap();
        assert!(check_metadata(
            &AccountInfo {
                key: &Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
                owner: &mpl_token_metadata::ID,
                lamports: Rc::new(RefCell::new(&mut 0)),
                data: Rc::new(RefCell::new(&mut buf[..])),
                executable: false,
                rent_epoch: 0,
            },
            &collection
        )
        .is_err());

        // Different collection
        let metadata = Metadata {
            programmable_config: None,
            key: Key::MetadataV1,
            update_authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            name: "".to_string(),
            symbol: "".to_string(),
            uri: "".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            primary_sale_happened: true,
            is_mutable: true,
            edition_nonce: Some(255),
            token_standard: None,
            collection: Some(Collection {
                verified: true,
                key: Pubkey::new_unique(),
            }),
            uses: None,
            collection_details: None,
        };
        let mut buf = vec![];
        metadata.serialize(&mut buf).unwrap();
        assert!(check_metadata(
            &AccountInfo {
                key: &Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
                owner: &mpl_token_metadata::ID,
                lamports: Rc::new(RefCell::new(&mut 0)),
                data: Rc::new(RefCell::new(&mut buf[..])),
                executable: false,
                rent_epoch: 0,
            },
            &collection
        )
        .is_err());

        // No collection
        let metadata = Metadata {
            programmable_config: None,
            key: Key::MetadataV1,
            update_authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            name: "".to_string(),
            symbol: "".to_string(),
            uri: "".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            primary_sale_happened: true,
            is_mutable: true,
            edition_nonce: Some(255),
            token_standard: None,
            collection: None,
            uses: None,
            collection_details: None,
        };
        let mut buf = vec![];
        metadata.serialize(&mut buf).unwrap();
        assert!(check_metadata(
            &AccountInfo {
                key: &Pubkey::new_unique(),
                is_signer: false,
                is_writable: true,
                owner: &mpl_token_metadata::ID,
                lamports: Rc::new(RefCell::new(&mut 0)),
                data: Rc::new(RefCell::new(&mut buf[..])),
                executable: false,
                rent_epoch: 0,
            },
            &collection
        )
        .is_err());
    }
}
