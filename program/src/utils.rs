use crate::state::schedule::Schedule;

use {
    solana_program::{hash::hashv, pubkey::Pubkey},
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

pub fn get_subdomain_key(ui_subdomain: String, parent: &Pubkey) -> Pubkey {
    let domain = format!("\0{}", ui_subdomain);
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &domain).as_bytes()])
        .as_ref()
        .to_vec();

    let (name_account_key, _) =
        get_seeds_and_key(&spl_name_service::ID, hashed_name, None, Some(parent));
    name_account_key
}

pub fn get_subdomain_reverse(ui_subdomain: String, parent: &Pubkey) -> Pubkey {
    let subdomain_key = get_subdomain_key(ui_subdomain, parent);
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + &subdomain_key.to_string()).as_bytes()])
        .as_ref()
        .to_vec();

    let (name_account_key, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed_name,
        Some(&name_auctioning::processor::CENTRAL_STATE),
        Some(parent),
    );
    name_account_key
}

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

    assert_eq!(get_domain_price("\01".to_string(), &schedule), 100);
    assert_eq!(get_domain_price("\011".to_string(), &schedule), 90);
    assert_eq!(get_domain_price("\0111".to_string(), &schedule), 80);
    assert_eq!(get_domain_price("\01111".to_string(), &schedule), 70);
    assert_eq!(get_domain_price("\011111".to_string(), &schedule), 60);
    assert_eq!(get_domain_price("\0111111".to_string(), &schedule), 60);
    assert_eq!(get_domain_price("\01111111111".to_string(), &schedule), 60);

    assert_eq!(get_domain_price("\0😀".to_string(), &schedule), 100);
    assert_eq!(get_domain_price("\01😀".to_string(), &schedule), 90);
    assert_eq!(get_domain_price("\01😀1".to_string(), &schedule), 80);
    assert_eq!(get_domain_price("\011😀1".to_string(), &schedule), 70);
    assert_eq!(get_domain_price("\0111😀1".to_string(), &schedule), 60);
    assert_eq!(get_domain_price("\011😀111".to_string(), &schedule), 60);
    assert_eq!(get_domain_price("\011😀1111111".to_string(), &schedule), 60);
}
