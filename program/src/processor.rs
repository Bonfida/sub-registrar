use {
    borsh::BorshDeserialize,
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
        pubkey::Pubkey,
    },
};

use crate::instruction::ProgramInstruction;

pub mod close_registry;
pub mod create_registry;
pub mod edit_registry;
pub mod register;
pub mod unregister;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = FromPrimitive::from_u8(instruction_data[0])
            .ok_or(ProgramError::InvalidInstructionData)?;
        let instruction_data = &instruction_data[1..];
        msg!("Instruction unpacked");

        match instruction {
            ProgramInstruction::CreateRegistry => {
                msg!("[+] Instruction: Create registry Instruction");
                let params = create_registry::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_registry::process(program_id, accounts, params)?;
            }
            ProgramInstruction::EditRegistry => {
                msg!("[+] Instruction: Edit registry Instruction");
                let params = edit_registry::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                edit_registry::process(program_id, accounts, params)?;
            }
            ProgramInstruction::Register => {
                msg!("[+] Instruction: Register Instruction");
                let params = register::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                register::process(program_id, accounts, params)?;
            }
            ProgramInstruction::Unregister => {
                msg!("[+] Instruction: Unregister Instruction");
                let params = unregister::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                unregister::process(program_id, accounts, params)?;
            }
            ProgramInstruction::CloseRegistry => {
                msg!("[+] Instruction: Close registry Instruction");
                let params = close_registry::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                close_registry::process(program_id, accounts, params)?;
            }
        }

        Ok(())
    }
}
