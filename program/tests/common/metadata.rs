use solana_program::program_option::COption;

use {
    mpl_token_metadata::state::{Collection, Data, Key, Metadata, TokenStandard},
    solana_program::{pubkey, pubkey::Pubkey},
    spl_token::state::Account,
};

// Example
// https://explorer.solana.com/address/BM6UwALQkF5yMv85Nv5YiVNVs5wgiRX2qTRPGbCSbvfF/metadata
pub const NFT_METADATA_KEY: Pubkey = pubkey!("A6GfBseUKrNZJy5y3WYGwvmLrdTMP1RoiXnrBoJABwDu");
pub const COLLECTION_KEY: Pubkey = pubkey!("E5ZnBpH9DYcxRkumKdS4ayJ3Ftb6o3E8wSbXw4N92GWg");
pub const NFT_MINT: Pubkey = pubkey!("BM6UwALQkF5yMv85Nv5YiVNVs5wgiRX2qTRPGbCSbvfF");
pub const UPDATE_AUTH: Pubkey = pubkey!("DL834WsTySeC2mJ5Wu9Unn2rYb6Abrot9P1b1Gq1XUVX");

pub fn get_metadata() -> Metadata {
    Metadata {
        key: Key::MetadataV1,
        update_authority: UPDATE_AUTH,
        mint: NFT_MINT,
        data: Data {
            name: "".to_owned(),
            symbol: "".to_owned(),
            uri: "".to_owned(),
            seller_fee_basis_points: 0,
            creators: None,
        },
        primary_sale_happened: true,
        is_mutable: true,
        edition_nonce: Some(255),
        token_standard: Some(TokenStandard::NonFungible),
        collection: Some(Collection {
            key: COLLECTION_KEY,
            verified: true,
        }),
        uses: None,
        collection_details: None,
    }
}

pub fn get_nft_account(owner: &Pubkey) -> Account {
    Account {
        mint: NFT_MINT,
        owner: *owner,
        amount: 1,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
}
