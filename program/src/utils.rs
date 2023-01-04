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
