use crate::error::SubRegisterError;

use super::schedule;

use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize, PartialEq, Debug, Eq, Default)]
pub struct Registrar {
    pub tag: super::Tag,
    pub nonce: u8,
    pub authority: Pubkey,
    pub fee_account: Pubkey,
    pub mint: Pubkey,
    pub domain_account: Pubkey,
    pub total_sub_created: u64,
    pub nft_gated_collection: Option<Pubkey>,
    pub max_nft_mint: u8,
    pub allow_revoke: bool,
    pub price_schedule: schedule::Schedule,
}

impl Registrar {
    pub const SEEDS: &'static [u8; 9] = b"registrar";

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        authority: &Pubkey,
        fee_account: &Pubkey,
        mint: &Pubkey,
        domain_account: &Pubkey,
        price_schedule: schedule::Schedule,
        nonce: u8,
        nft_gated_collection: Option<Pubkey>,
        max_nft_mint: u8,
        allow_revoke: bool,
    ) -> Self {
        Self {
            tag: super::Tag::Registrar,
            nonce,
            authority: *authority,
            fee_account: *fee_account,
            mint: *mint,
            domain_account: *domain_account,
            total_sub_created: 0,
            price_schedule,
            nft_gated_collection,
            max_nft_mint,
            allow_revoke,
        }
    }

    pub fn find_key(
        domain_account: &Pubkey,
        authority: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Registrar::SEEDS,
                &domain_account.to_bytes(),
                &authority.to_bytes(),
            ],
            program_id,
        )
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(a: &AccountInfo, tag: super::Tag) -> Result<Registrar, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != tag as u8 && data[0] != super::Tag::Uninitialized as u8 {
            return Err(SubRegisterError::DataTypeMismatch.into());
        }
        let result = Registrar::deserialize(&mut data)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc, u8};

    use super::*;
    use crate::state::Tag;

    #[test]
    fn test_from_account_info() {
        let registrar: Registrar = Registrar {
            tag: Tag::Registrar,
            ..Registrar::default()
        };
        let closed_registrar: Registrar = Registrar {
            tag: Tag::ClosedRegistrar,
            ..Registrar::default()
        };
        let mut buf: Vec<u8> = vec![0; registrar.borsh_len()];
        registrar.save(&mut buf[..]);

        let des = Registrar::from_account_info(
            &AccountInfo {
                data: Rc::new(RefCell::new(&mut buf[..])),
                key: &Pubkey::default(),
                is_signer: false,
                is_writable: false,
                lamports: Rc::new(RefCell::new(&mut 0)),
                owner: &Pubkey::default(),
                executable: false,
                rent_epoch: 0,
            },
            Tag::Registrar,
        )
        .unwrap();
        assert_eq!(registrar, des);

        let res = Registrar::from_account_info(
            &AccountInfo {
                data: Rc::new(RefCell::new(&mut buf[..])),
                key: &Pubkey::default(),
                is_signer: false,
                is_writable: false,
                lamports: Rc::new(RefCell::new(&mut 0)),
                owner: &Pubkey::default(),
                executable: false,
                rent_epoch: 0,
            },
            Tag::ClosedRegistrar,
        );
        assert!(res.is_err());

        let mut buf: Vec<u8> = vec![0; registrar.borsh_len()];
        closed_registrar.save(&mut buf);

        let des = Registrar::from_account_info(
            &AccountInfo {
                data: Rc::new(RefCell::new(&mut buf[..])),
                key: &Pubkey::default(),
                is_signer: false,
                is_writable: false,
                lamports: Rc::new(RefCell::new(&mut 0)),
                owner: &Pubkey::default(),
                executable: false,
                rent_epoch: 0,
            },
            Tag::ClosedRegistrar,
        )
        .unwrap();
        assert_eq!(closed_registrar, des);

        let res = Registrar::from_account_info(
            &AccountInfo {
                data: Rc::new(RefCell::new(&mut buf[..])),
                key: &Pubkey::default(),
                is_signer: false,
                is_writable: false,
                lamports: Rc::new(RefCell::new(&mut 0)),
                owner: &Pubkey::default(),
                executable: false,
                rent_epoch: 0,
            },
            Tag::Registrar,
        );
        assert!(res.is_err());
    }
}
