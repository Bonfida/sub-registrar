use std::str::FromStr;

use async_trait::async_trait;
use borsh::BorshSerialize;
use solana_program::clock::Clock;
use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext, ProgramTestError};
use solana_sdk::account::Account;
use solana_sdk::signature::Signer;
use solana_sdk::{signature::Keypair, transaction::Transaction};
use spl_token::state::Mint;
use sub_register::state::schedule::{Price, Schedule};

// Utils
pub async fn sign_send_instructions(
    ctx: &mut ProgramTestContext,
    instructions: Vec<Instruction>,
    signers: Vec<&Keypair>,
) -> Result<(), BanksClientError> {
    let slot = ctx.banks_client.get_root_slot().await?;
    ctx.warp_to_slot(slot + 1).unwrap();
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&ctx.payer.pubkey()));
    let mut payer_signers = vec![&ctx.payer];
    for s in signers {
        payer_signers.push(s);
    }
    transaction.partial_sign(&payer_signers, ctx.last_blockhash);
    ctx.banks_client.process_transaction(transaction).await
}

pub fn mint_bootstrap(
    address: Option<&str>,
    decimals: u8,
    program_test: &mut ProgramTest,
    mint_authority: &Pubkey,
) -> (Pubkey, Mint) {
    let address = address
        .map(|s| Pubkey::from_str(s).unwrap())
        .unwrap_or_else(Pubkey::new_unique);
    let mint_info = Mint {
        mint_authority: Some(*mint_authority).into(),
        supply: u32::MAX.into(),
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    };
    let mut data = [0; Mint::LEN];
    mint_info.pack_into_slice(&mut data);
    program_test.add_account(
        address,
        Account {
            lamports: u32::MAX.into(),
            data: data.into(),
            owner: spl_token::ID,
            executable: false,
            ..Account::default()
        },
    );
    (address, mint_info)
}

const CHARSET: &str = "1234567890";

pub fn random_string() -> String {
    random_string::generate(10, CHARSET)
}

pub fn convert_schedule(schedule: Schedule) -> Vec<Vec<u64>> {
    let mut res = vec![];
    for x in schedule {
        res.push(vec![x.length, x.price])
    }
    res
}

pub fn serialize_price_schedule(price_schedule: &[Price]) -> Vec<u8> {
    let mut data: Vec<u8> = vec![];
    price_schedule.serialize(&mut data).unwrap();
    data
}

#[async_trait]
pub trait ProgramTestContextExtended {
    async fn warp_forward(&mut self, timestamp: i64) -> Result<(), ProgramTestError>;
}

#[async_trait]
impl ProgramTestContextExtended for ProgramTestContext {
    async fn warp_forward(&mut self, duration: i64) -> Result<(), ProgramTestError> {
        // Adapted from https://github.com/halbornteam/solana-test-framework/blob/45e5e43aa1e321ead8ff6cd6e5d6b45e1921733d/src/extensions/program_test_context.rs#L31
        const NANOSECONDS_IN_SECOND: i64 = 1_000_000_000;

        let mut clock: Clock = self.banks_client.get_sysvar().await.unwrap();
        let current_slot = clock.slot;
        clock.unix_timestamp += duration;

        if duration <= 0 {
            println!("Timestamp incorrect. Cannot set time backwards.");
            return Err(ProgramTestError::InvalidWarpSlot);
        }

        let ns_per_slot = self.genesis_config().ns_per_slot();
        let timestamp_diff_ns = duration
            .checked_mul(NANOSECONDS_IN_SECOND) //convert from s to ns
            .expect("Problem with timestamp diff calculation.")
            as u128;

        let slots = timestamp_diff_ns
            .checked_div(ns_per_slot)
            .expect("Problem with slots from timestamp calculation.") as u64;

        self.set_sysvar(&clock);
        self.warp_to_slot(current_slot + slots)?;
        Ok(())
    }
}
