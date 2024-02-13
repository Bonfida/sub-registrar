use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, instruction::AccountMeta, msg,
    program::invoke_signed, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    rent::Rent, system_instruction::create_account, sysvar::Sysvar,
};

use crate::state::registry::Registrar;

#[allow(missing_docs)]
pub struct Cpi {}

impl Cpi {
    #[allow(missing_docs)]
    pub fn create_account<'a>(
        program_id: &Pubkey,
        system_program: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        account_to_create: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        space: usize,
    ) -> ProgramResult {
        let create_state_instruction = create_account(
            fee_payer.key,
            account_to_create.key,
            Rent::get()?.minimum_balance(space),
            space as u64,
            program_id,
        );

        invoke_signed(
            &create_state_instruction,
            &[
                system_program.clone(),
                fee_payer.clone(),
                account_to_create.clone(),
            ],
            &[signer_seeds],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn allocate_and_create_token_account<'a>(
        token_account_owner: &Pubkey,
        spl_token_program: &AccountInfo<'a>,
        payer_info: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        token_account: &AccountInfo<'a>,
        mint_account: &AccountInfo<'a>,
        rent_account: &AccountInfo<'a>,
        system_program_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        msg!("Initializing token account");
        let size = spl_token::state::Account::LEN;
        let required_lamports = Rent::get()?.minimum_balance(size);
        let ix_allocate = create_account(
            payer_info.key,
            token_account.key,
            required_lamports,
            size as u64,
            &spl_token::ID,
        );
        invoke_signed(
            &ix_allocate,
            &[
                system_program_info.clone(),
                payer_info.clone(),
                token_account.clone(),
            ],
            &[signer_seeds],
        )?;
        let ix_initialize = spl_token::instruction::initialize_account2(
            &spl_token::ID,
            token_account.key,
            mint_account.key,
            token_account_owner,
        )?;
        invoke_signed(
            &ix_initialize,
            &[
                spl_token_program.clone(),
                token_account.clone(),
                mint_account.clone(),
                rent_account.clone(),
            ],
            &[signer_seeds],
        )?;
        Ok(())
    }

    pub fn transfer_subdomain<'a>(
        registrar: &Registrar,
        registrar_account: &AccountInfo<'a>,
        sub_account: &AccountInfo<'a>,
        parent_account: &AccountInfo<'a>,
        name_class: &AccountInfo<'a>,
        spl_name_service: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let mut ix = spl_name_service::instruction::transfer(
            spl_name_service::ID,
            *registrar_account.key,
            *sub_account.key,
            *registrar_account.key,
            None,
        )?;
        ix.accounts
            .push(AccountMeta::new_readonly(*name_class.key, false));
        ix.accounts
            .push(AccountMeta::new_readonly(*parent_account.key, false));
        let seeds: &[&[u8]] = &[
            Registrar::SEEDS,
            &registrar.domain_account.to_bytes(),
            &[registrar.nonce],
        ];
        invoke_signed(
            &ix,
            &[
                spl_name_service.clone(),
                registrar_account.clone(),
                sub_account.clone(),
                parent_account.clone(),
                name_class.clone(),
                registrar_account.clone(),
            ],
            &[seeds],
        )?;

        Ok(())
    }
}
