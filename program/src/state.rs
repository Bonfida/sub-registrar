use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{pubkey, pubkey::Pubkey},
};

pub mod registry;
pub mod schedule;

pub const ROOT_DOMAIN_ACCOUNT: Pubkey = pubkey!("58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx");
pub const NAME_AUCTIONING: Pubkey = pubkey!("jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR");

// 5% fee
pub const FEE_PCT: u64 = 5;
// Fee account
// TODO: change to real address
pub const FEE_ACC_OWNER: Pubkey = pubkey!("G9tP6ZonwNj2qTdPpCrTsrCQgDovppxjCddfidNwFq5n");

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq, Debug, Eq)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    Registrar,
    ClosedRegistrar,
}
