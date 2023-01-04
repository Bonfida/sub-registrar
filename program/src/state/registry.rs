use crate::error::SubRegisterError;

use super::schedule;

use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Registry {
    pub tag: super::Tag,
    pub nonce: u8,
    pub authority: Pubkey,
    pub fee_account: Pubkey,
    pub mint: Pubkey,
    pub domain_account: Pubkey,
    pub total_sub_created: u64,
    pub price_schedule: schedule::Schedule,
}

impl Registry {
    pub const SEEDS: &[u8; 8] = b"registry";

    pub fn new(
        authority: &Pubkey,
        fee_account: &Pubkey,
        mint: &Pubkey,
        domain_account: &Pubkey,
        price_schedule: schedule::Schedule,
        nonce: u8,
    ) -> Self {
        Self {
            tag: super::Tag::Registry,
            nonce,
            authority: *authority,
            fee_account: *fee_account,
            mint: *mint,
            domain_account: *domain_account,
            total_sub_created: 0,
            price_schedule: price_schedule,
        }
    }

    pub fn find_key(
        domain_account: &Pubkey,
        authority: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Registry::SEEDS,
                &domain_account.to_bytes(),
                &authority.to_bytes(),
            ],
            program_id,
        )
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(a: &AccountInfo, tag: super::Tag) -> Result<Registry, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != tag as u8 && data[0] != super::Tag::Uninitialized as u8 {
            return Err(SubRegisterError::DataTypeMismatch.into());
        }
        let result = Registry::deserialize(&mut data)?;
        Ok(result)
    }
}
