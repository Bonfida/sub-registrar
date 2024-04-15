use super::Tag;
use crate::error::SubRegisterError;
use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

// MintRecords are used to keep track of how many domains were minted via a specific NFT ownership.
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, BorshSize)]
pub struct MintRecord {
    pub tag: Tag,
    // How many subdomains have been minted so far for this NFT
    pub count: u8,
    // The mint of the NFT
    pub mint: Pubkey,
}

impl MintRecord {
    pub const SEEDS: &'static [u8; 15] = b"nft_mint_record";

    pub fn new(mint: &Pubkey) -> Self {
        Self {
            tag: Tag::MintRecord,
            count: 0,
            mint: *mint,
        }
    }

    pub fn find_key(mint: &Pubkey, registrar: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[MintRecord::SEEDS, &registrar.to_bytes(), &mint.to_bytes()],
            program_id,
        )
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(a: &AccountInfo, tag: super::Tag) -> Result<MintRecord, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != tag as u8 && data[0] != super::Tag::Uninitialized as u8 {
            return Err(SubRegisterError::DataTypeMismatch.into());
        }
        let result = MintRecord::deserialize(&mut data)?;
        Ok(result)
    }
}
