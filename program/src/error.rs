use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum SubRegisterError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account is uninitialized")]
    Uninitialized,
    #[error("Invalid name account")]
    WrongNameAccount,
    #[error("Cannot close registry")]
    CannotCloseRegistry,
    #[error("Numerical overflow")]
    Overflow,
    #[error("Invalid subdomain")]
    InvalidSubdomain,
    #[error("Must hold one NFT")]
    MustHoldOneNFt,
    #[error("Must provide NFT account")]
    MustProvideNft,
    #[error("Must provide NFT metadata account")]
    MustProvideNftMetadata,
    #[error("NFT must have a collection")]
    MustHaveCollection,
    #[error("Invalid collection")]
    InvalidCollection,
    #[error("Must provide mint record account")]
    MustProvideNftMintRecord,
    #[error("Mint limit reach")]
    MintLimitReached,
    #[error("Cannot revoke")]
    CannotRevoke,
    #[error("Missing account")]
    MissingAccount,
    #[error("Missing mint record")]
    MissingMintRecord,
    #[error("Wrong mint record")]
    WrongMintRecord,
    #[error("The revoked domain is still not expired to protect from impersonation")]
    RevokedSubdomainNotExpired,
    #[error("The proposed expiry delay for revoked subdomains is too low")]
    RevokeExpiryDelayTooLow,
    #[error("Wrong mint")]
    WrongMint,
}

impl From<SubRegisterError> for ProgramError {
    fn from(e: SubRegisterError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SubRegisterError {
    fn type_of() -> &'static str {
        "SubRegisterError"
    }
}
