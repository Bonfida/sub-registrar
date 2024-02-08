use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, BorshSize, PartialEq, Eq, Debug)]
pub struct Price {
    pub length: u64,
    pub price: u64,
}

// Assumes the `Schedule` is ordered in ascending order on the `Price.length`
pub type Schedule = Vec<Price>;
