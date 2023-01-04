use crate::{error::SubRegisterError, processor::Processor};

use {
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, decode_error::DecodeError, entrypoint::ProgramResult, msg,
        program_error::PrintProgramError, pubkey::Pubkey,
    },
};

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

/// The entrypoint to the program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<SubRegisterError>();
        return Err(error);
    }
    Ok(())
}

impl PrintProgramError for SubRegisterError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            SubRegisterError::AlreadyInitialized => {
                msg!("[+] Error: This account is already initialized")
            }
            SubRegisterError::DataTypeMismatch => msg!("[+] Error: Data type mismatch"),
            SubRegisterError::WrongOwner => msg!("[+] Error: Wrong account owner"),
            SubRegisterError::Uninitialized => msg!("[+] Error: Account is uninitialized"),
            SubRegisterError::WrongNameAccount => msg!("[+] Error: Invalid name account"),
            SubRegisterError::CannotCloseRegistry => msg!("[+] Error: Cannot close registry"),
            SubRegisterError::Overflow => msg!("[+] Error: Numerical overflow"),
            SubRegisterError::InvalidSubdomain => msg!("[+] Error: Invalid subdomain"),
        }
    }
}
