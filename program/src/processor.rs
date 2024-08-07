use {
    borsh::BorshDeserialize,
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
        pubkey::Pubkey,
    },
};

use crate::instruction::ProgramInstruction;

pub mod admin_register;
pub mod admin_revoke;
pub mod close_registrar;
pub mod create_registrar;
pub mod delete_subdomain_record;
pub mod edit_registrar;
pub mod nft_owner_revoke;
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
            ProgramInstruction::CreateRegistrar => {
                msg!("[+] Instruction: Create registrar Instruction");
                let params = create_registrar::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_registrar::process(program_id, accounts, params)?;
            }
            ProgramInstruction::EditRegistrar => {
                msg!("[+] Instruction: Edit registrar Instruction");
                let params = edit_registrar::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                edit_registrar::process(program_id, accounts, params)?;
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
            ProgramInstruction::CloseRegistrar => {
                msg!("[+] Instruction: Close registrar Instruction");
                let params = close_registrar::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                close_registrar::process(program_id, accounts, params)?;
            }
            ProgramInstruction::AdminRegister => {
                msg!("[+] Instruction: Admin register Instruction");
                let params = admin_register::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                admin_register::process(program_id, accounts, params)?;
            }
            ProgramInstruction::DeleteSubdomainRecord => {
                msg!("[+] Instruction: Delete sub record Instruction");
                let params = delete_subdomain_record::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                delete_subdomain_record::process(program_id, accounts, params)?;
            }
            ProgramInstruction::AdminRevoke => {
                msg!("[+] Instruction: Admin revoke instruction");
                let params = admin_revoke::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                admin_revoke::process(program_id, accounts, params)?;
            }
            ProgramInstruction::NftOwnerRevoke => {
                msg!("[+] Instruction: NFT owner revoke instruction");
                let params = nft_owner_revoke::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                nft_owner_revoke::process(program_id, accounts, params)?;
            }
        }

        Ok(())
    }
}
