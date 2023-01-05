pub use crate::processor::{
    admin_register, close_registry, create_registry, edit_registry, register, unregister,
};
use {
    bonfida_utils::InstructionsAccount,
    borsh::{BorshDeserialize, BorshSerialize},
    num_derive::FromPrimitive,
    solana_program::{instruction::Instruction, pubkey::Pubkey},
};
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Create registry
    ///
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account           |
    /// | 1     | ✅        | ❌      | The registry account                 |
    /// | 2     | ✅        | ❌      | The domain account                   |
    /// | 3     | ✅        | ✅      | The owner of the domain name account |
    /// | 4     | ✅        | ✅      | The fee payer account                |
    /// | 5     | ❌        | ❌      | The SPL name service program ID      |
    CreateRegistry,
    /// Edit a registry
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ❌        | ❌      | The system program account |
    /// | 1     | ✅        | ✅      | The fee payer account      |
    /// | 2     | ✅        | ❌      | The registry to edit       |
    EditRegistry,
    /// Register a subdomain
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account           |
    /// | 1     | ❌        | ❌      | The SPL token program account        |
    /// | 2     | ❌        | ❌      | The SPL name service program account |
    /// | 3     | ❌        | ❌      | The rent sysvar account              |
    /// | 4     | ❌        | ❌      | The name auctioning program account  |
    /// | 5     | ❌        | ❌      | The .sol root domain                 |
    /// | 6     | ❌        | ❌      | The reverse lookup class accoutn     |
    /// | 7     | ✅        | ❌      | The fee account of the registry      |
    /// | 8     | ✅        | ❌      |                                      |
    /// | 9     | ✅        | ❌      |                                      |
    /// | 10    | ✅        | ❌      |                                      |
    /// | 11    | ✅        | ❌      |                                      |
    /// | 12    | ✅        | ❌      |                                      |
    /// | 13    | ✅        | ✅      | The fee payer account                |
    Register,
    /// Unregister a subdomain
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account           |
    /// | 1     | ❌        | ❌      | The SPL name service program account |
    /// | 2     | ✅        | ❌      |                                      |
    /// | 3     | ✅        | ❌      |                                      |
    /// | 4     | ✅        | ✅      | The fee payer account                |
    Unregister,
    /// Close a registry account
    ///
    /// | Index | Writable | Signer | Description                              |
    /// | -------------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account               |
    /// | 1     | ✅        | ❌      | The registry account                     |
    /// | 2     | ✅        | ❌      | The domain account                       |
    /// | 3     | ❌        | ❌      | The new owner of the domain name account |
    /// | 4     | ✅        | ❌      | The lamports target                      |
    /// | 5     | ✅        | ✅      | The authority of the registry            |
    /// | 6     | ❌        | ❌      | The SPL name service program ID          |
    CloseRegistry,
    /// Allow the authority of a `Registry` to register a subdomain without token transfer
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account           |
    /// | 1     | ❌        | ❌      | The SPL token program account        |
    /// | 2     | ❌        | ❌      | The SPL name service program account |
    /// | 3     | ❌        | ❌      | The rent sysvar account              |
    /// | 4     | ❌        | ❌      | The name auctioning program account  |
    /// | 5     | ❌        | ❌      | The .sol root domain                 |
    /// | 6     | ❌        | ❌      | The reverse lookup class accoutn     |
    /// | 7     | ✅        | ❌      |                                      |
    /// | 8     | ✅        | ❌      |                                      |
    /// | 9     | ✅        | ❌      |                                      |
    /// | 10    | ✅        | ❌      |                                      |
    /// | 11    | ✅        | ✅      | The fee payer account                |
    AdminRegister,
}
pub fn create_registry(
    accounts: create_registry::Accounts<Pubkey>,
    params: create_registry::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::CreateRegistry as u8, params)
}
pub fn edit_registry(
    accounts: edit_registry::Accounts<Pubkey>,
    params: edit_registry::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::EditRegistry as u8, params)
}
pub fn register(accounts: register::Accounts<Pubkey>, params: register::Params) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::Register as u8, params)
}
pub fn unregister(
    accounts: unregister::Accounts<Pubkey>,
    params: unregister::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::Unregister as u8, params)
}
pub fn close_registry(
    accounts: close_registry::Accounts<Pubkey>,
    params: close_registry::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::CloseRegistry as u8, params)
}
pub fn admin_register(
    accounts: admin_register::Accounts<Pubkey>,
    params: admin_register::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::AdminRegister as u8, params)
}
