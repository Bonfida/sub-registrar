use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{pubkey, pubkey::Pubkey},
};

pub mod mint_record;
pub mod registry;
pub mod schedule;
pub mod subdomain_record;

pub const ROOT_DOMAIN_ACCOUNT: Pubkey = sns_registrar::constants::ROOT_DOMAIN_ACCOUNT;

// 5% fee
pub const FEE_PCT: u64 = 5;
// Fee account
pub const FEE_ACC_OWNER: Pubkey = pubkey!("5D2zKog251d6KPCyFyLMt3KroWwXXPWSgTPyhV22K2gR");

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq, Debug, Eq)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    Registrar,
    ClosedRegistrar,
    SubRecord,
    ClosedSubRecord,
    MintRecord,
    RevokedSubRecord,
}

impl Default for Tag {
    fn default() -> Self {
        Self::Uninitialized
    }
}

impl Tag {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Uninitialized),
            1 => Some(Self::Registrar),
            2 => Some(Self::ClosedRegistrar),
            3 => Some(Self::SubRecord),
            4 => Some(Self::ClosedSubRecord),
            5 => Some(Self::MintRecord),
            6 => Some(Self::RevokedSubRecord),
            _ => None,
        }
    }
}
