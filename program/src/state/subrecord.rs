use super::Tag;
use crate::error::SubRegisterError;
use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, BorshSize)]
pub struct SubRecord {
    pub tag: Tag,
    pub registrar: Pubkey,
    pub mint_record: Option<Pubkey>,
}

impl SubRecord {
    pub const SEEDS: &'static [u8; 9] = b"subrecord";

    pub fn new(registrar: Pubkey) -> Self {
        Self {
            tag: Tag::SubRecord,
            registrar,
            mint_record: None,
        }
    }

    pub fn find_key(domain_account: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[SubRecord::SEEDS, &domain_account.to_bytes()], program_id)
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(a: &AccountInfo, tag: super::Tag) -> Result<SubRecord, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != tag as u8 && data[0] != super::Tag::Uninitialized as u8 {
            return Err(SubRegisterError::DataTypeMismatch.into());
        }
        let result = SubRecord::deserialize(&mut data)?;
        Ok(result)
    }
}

impl Default for SubRecord {
    fn default() -> Self {
        Self::new(Pubkey::default())
    }
}
