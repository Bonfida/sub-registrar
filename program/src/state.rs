use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{pubkey, pubkey::Pubkey},
};

pub mod registry;
pub mod schedule;

pub const ROOT_DOMAIN_ACCOUNT: Pubkey = pubkey!("58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx");
pub const NAME_AUCTIONING: Pubkey = pubkey!("jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR");

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq, Debug, Eq)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    Registry,
    ClosedRegistry,
}
