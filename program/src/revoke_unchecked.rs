//! Revoke unchecked

use crate::{
    cpi::Cpi,
    error::SubRegisterError,
    state::{mint_record::MintRecord, registry::Registrar, subdomain_record::SubDomainRecord, Tag},
};

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
};

// All accounts checks must be done before calling this function!
#[allow(clippy::too_many_arguments)]
pub fn revoke_unchecked<'a>(
    mut registrar: Registrar,
    mut sub_record: SubDomainRecord,
    mint_record: Option<MintRecord>,
    registrar_account: &AccountInfo<'a>,
    subdomain_account: &AccountInfo<'a>,
    parent_domain_account: &AccountInfo<'a>,
    name_class_account: &AccountInfo<'a>,
    spl_name_service_account: &AccountInfo<'a>,
    sub_record_account: &AccountInfo<'a>,
    lamport_target_account: &AccountInfo<'a>,
    mint_record_account: Option<&AccountInfo<'a>>,
) -> ProgramResult {
    // Transfer to registrar
    Cpi::transfer_subdomain(
        &registrar,
        registrar_account,
        subdomain_account,
        parent_domain_account,
        name_class_account,
        spl_name_service_account,
    )?;

    // Unregister domain
    let seeds: &[&[u8]] = &[
        Registrar::SEEDS,
        &registrar.domain_account.to_bytes(),
        &registrar.authority.to_bytes(),
        &[registrar.nonce],
    ];
    let ix = spl_name_service::instruction::delete(
        spl_name_service::ID,
        *subdomain_account.key,
        *registrar_account.key,
        *lamport_target_account.key,
    )?;
    invoke_signed(
        &ix,
        &[
            spl_name_service_account.clone(),
            subdomain_account.clone(),
            registrar_account.clone(),
            lamport_target_account.clone(),
        ],
        &[seeds],
    )?;

    // Close subrecord account
    sub_record.tag = Tag::ClosedSubRecord;
    sub_record.save(&mut sub_record_account.data.borrow_mut());

    // Zero out lamports of subrecord account
    let mut sub_record_lamports = sub_record_account.lamports.borrow_mut();
    let mut target_lamports = lamport_target_account.lamports.borrow_mut();

    **target_lamports += **sub_record_lamports;
    **sub_record_lamports = 0;

    // Decrement mint record count
    if let Some(mut mint_record) = mint_record {
        mint_record.count = mint_record
            .count
            .checked_sub(1)
            .ok_or(SubRegisterError::Overflow)?;
        mint_record.save(&mut mint_record_account.unwrap().data.borrow_mut());
    }

    // Decrement nb sub created
    registrar.total_sub_created = registrar
        .total_sub_created
        .checked_sub(1)
        .ok_or(SubRegisterError::Overflow)?;

    // Serialize state
    registrar.save(&mut registrar_account.data.borrow_mut());
    Ok(())
}
