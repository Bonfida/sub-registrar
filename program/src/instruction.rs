pub use crate::processor::{
    admin_register, close_registrar, create_registrar, delete_subrecord, edit_registrar, register,
    unregister,
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
    /// Create registrar
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account           |
    /// | 1     | ✅        | ❌      | The registrar account                |
    /// | 2     | ✅        | ❌      | The domain account                   |
    /// | 3     | ✅        | ✅      | The owner of the domain name account |
    /// | 4     | ✅        | ✅      | The fee payer account                |
    /// | 5     | ❌        | ❌      | The SPL name service program ID      |
    CreateRegistrar,
    /// Edit a registrar
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ❌        | ❌      | The system program account |
    /// | 1     | ✅        | ✅      | The fee payer account      |
    /// | 2     | ✅        | ❌      | The registry to edit       |
    EditRegistrar,
    /// Register a subdomain
    ///
    /// | Index | Writable | Signer | Description                                             |
    /// | ----------------------------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account                              |
    /// | 1     | ❌        | ❌      | The SPL token program account                           |
    /// | 2     | ❌        | ❌      | The SPL name service program account                    |
    /// | 3     | ❌        | ❌      | The rent sysvar account                                 |
    /// | 4     | ❌        | ❌      | The name auctioning program account                     |
    /// | 5     | ❌        | ❌      | The .sol root domain                                    |
    /// | 6     | ❌        | ❌      | The reverse lookup class accoutn                        |
    /// | 7     | ✅        | ❌      | The fee account of the registry                         |
    /// | 8     | ✅        | ❌      |                                                         |
    /// | 9     | ✅        | ❌      |                                                         |
    /// | 10    | ✅        | ❌      |                                                         |
    /// | 11    | ✅        | ❌      |                                                         |
    /// | 12    | ✅        | ❌      |                                                         |
    /// | 13    | ✅        | ✅      | The fee payer account                                   |
    /// | 14    | ✅        | ❌      |                                                         |
    /// | 15    | ✅        | ❌      | The subrecord account                                   |
    /// | 16    | ❌        | ❌      | Optional NFT account if Registrar is NFT gated          |
    /// | 17    | ❌        | ❌      | Optional NFT metadata account if Registrar is NFT gated |
    Register,
    /// Unregister a subdomain
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account           |
    /// | 1     | ❌        | ❌      | The SPL name service program account |
    /// | 2     | ✅        | ❌      | The registrar account                |
    /// | 3     | ✅        | ❌      | The subdomain account to unregister  |
    /// | 4     | ✅        | ❌      | The subrecord account                |
    /// | 5     | ✅        | ✅      | The fee payer account                |
    Unregister,
    /// Close a registrar account
    ///
    /// | Index | Writable | Signer | Description                              |
    /// | -------------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account               |
    /// | 1     | ✅        | ❌      | The registrar account                    |
    /// | 2     | ✅        | ❌      | The domain account                       |
    /// | 3     | ❌        | ❌      | The new owner of the domain name account |
    /// | 4     | ✅        | ❌      | The lamports target                      |
    /// | 5     | ✅        | ✅      | The authority of the registry            |
    /// | 6     | ❌        | ❌      | The SPL name service program ID          |
    CloseRegistrar,
    /// Allow the authority of a `Registrar` to register a subdomain without token transfer
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
    /// | 7     | ✅        | ❌      | The registrar account                |
    /// | 8     | ✅        | ❌      | The parent domain account            |
    /// | 9     | ✅        | ❌      | The subdomain account to create      |
    /// | 10    | ✅        | ❌      | The subdomain reverse account        |
    /// | 11    | ✅        | ❌      | The subrecord account                |
    /// | 12    | ✅        | ✅      | The fee payer account                |
    AdminRegister,
    /// Delete a subrecord account account
    ///
    /// | Index | Writable | Signer | Description            |
    /// | -------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The sub domain account |
    /// | 1     | ✅        | ❌      | The sub record account |
    /// | 2     | ✅        | ❌      | The lamports target    |
    DeleteSubrecord,
}
pub fn create_registrar(
    accounts: create_registrar::Accounts<Pubkey>,
    params: create_registrar::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::CreateRegistrar as u8, params)
}
pub fn edit_registrar(
    accounts: edit_registrar::Accounts<Pubkey>,
    params: edit_registrar::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::EditRegistrar as u8, params)
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
pub fn close_registrar(
    accounts: close_registrar::Accounts<Pubkey>,
    params: close_registrar::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::CloseRegistrar as u8, params)
}
pub fn admin_register(
    accounts: admin_register::Accounts<Pubkey>,
    params: admin_register::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::AdminRegister as u8, params)
}
pub fn delete_subrecord(
    accounts: delete_subrecord::Accounts<Pubkey>,
    params: delete_subrecord::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::DeleteSubrecord as u8, params)
}
