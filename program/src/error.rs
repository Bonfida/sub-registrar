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
