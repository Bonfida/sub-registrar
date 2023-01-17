use super::Tag;
use crate::error::SubRegisterError;
use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, BorshSize)]
pub struct NftMintRecord {
    pub tag: Tag,
    pub count: u8,
}

impl NftMintRecord {
    pub const SEEDS: &'static [u8; 15] = b"nft_mint_record";

    pub fn new() -> Self {
        Self {
            tag: Tag::NftMintRecord,
            count: 0,
        }
    }

    pub fn find_key(mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[NftMintRecord::SEEDS, &mint.to_bytes()], program_id)
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(
        a: &AccountInfo,
        tag: super::Tag,
    ) -> Result<NftMintRecord, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != tag as u8 && data[0] != super::Tag::Uninitialized as u8 {
            return Err(SubRegisterError::DataTypeMismatch.into());
        }
        let result = NftMintRecord::deserialize(&mut data)?;
        Ok(result)
    }
}

impl Default for NftMintRecord {
    fn default() -> Self {
        Self::new()
    }
}
