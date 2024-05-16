use super::Tag;
use crate::error::SubRegisterError;
use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

// After at least a week, a subdomain can be overwritten
pub const REVOKE_EXPIRY_DELAY_SECONDS_MIN: i64 = 604800;

// SubRecord are used to keep track of subs minted via a specific registrar
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, BorshSize)]
pub struct SubDomainRecord {
    pub tag: Tag,
    // The registrar the record belongs to
    pub registrar: Pubkey,
    // The subdomain key associated to this record
    pub sub_key: Pubkey,
    // If the record is associated to a NFT
    pub mint_record: Option<Pubkey>,
    // Expiry timestamp. From this timestamp on the subdomain
    // and subdomain record can be deleted
    pub expiry_timestamp: i64,
}

impl SubDomainRecord {
    pub const SEEDS: &'static [u8; 9] = b"subrecord";

    pub fn new(registrar: Pubkey, sub_key: Pubkey) -> Self {
        Self {
            tag: Tag::SubRecord,
            registrar,
            sub_key,
            mint_record: None,
            expiry_timestamp: i64::MAX,
        }
    }

    pub fn find_key(domain_account: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[SubDomainRecord::SEEDS, &domain_account.to_bytes()],
            program_id,
        )
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(
        a: &AccountInfo,
        tag: super::Tag,
    ) -> Result<SubDomainRecord, ProgramError> {
        Self::from_account_info_opt(a, Some(tag))
    }

    pub fn from_account_info_opt(
        a: &AccountInfo,
        tag: Option<super::Tag>,
    ) -> Result<SubDomainRecord, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if tag.map(|t| data[0] != t as u8).unwrap_or(false)
            && data[0] != super::Tag::Uninitialized as u8
        {
            return Err(SubRegisterError::DataTypeMismatch.into());
        }
        let result = SubDomainRecord::deserialize(&mut data)?;
        Ok(result)
    }
}
