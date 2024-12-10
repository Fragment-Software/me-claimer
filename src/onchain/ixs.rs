use rand::seq::SliceRandom;
use rand::thread_rng;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_sdk::native_token::{lamports_to_sol, sol_to_lamports};

use crate::jito::constants::JITO_BUNDLE_TIP_ACCOUNTS;

use super::constants::{
    ASSOCIATED_TOKEN_PROGRAM_ID, CLOSE_PUBKEY, ME_PUBKEY, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
};
use super::derive::derive_ata;
use super::typedefs::CreateAtaArgs;

pub struct Instructions {}

impl Instructions {
    pub fn create_ata(args: CreateAtaArgs) -> Instruction {
        Instruction {
            program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(args.funding_address, true),
                AccountMeta::new(args.associated_account_address, false),
                AccountMeta::new_readonly(args.wallet_address, false),
                AccountMeta::new_readonly(args.token_mint_address, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(args.token_program_id, false),
            ],
            data: vec![args.instruction],
        }
    }

    pub fn close_account(
        wallet_token_ata: &Pubkey,
        wallet_pubkey: &Pubkey,
        payer_pubkey: &Pubkey,
        rent: u64,
    ) -> [Instruction; 2] {
        let close_amount = sol_to_lamports(lamports_to_sol(rent) * (0.01 + 0.01 + 0.01 + 0.02));

        [
            spl_token::instruction::close_account(
                &TOKEN_PROGRAM_ID,
                wallet_token_ata,
                payer_pubkey,
                wallet_pubkey,
                &[wallet_pubkey],
            )
            .expect("Close ix to be valid"),
            solana_sdk::system_instruction::transfer(payer_pubkey, &CLOSE_PUBKEY, close_amount),
        ]
    }

    pub fn tip_ix(
        wallet_token_ata: &Pubkey,
        wallet_pubkey: &Pubkey,
        payer_pubkey: &Pubkey,
        jito_tip_amount: f64,
        rent: u64,
    ) -> [Instruction; 2] {
        let mut rng = thread_rng();
        let (me_ata, _) = derive_ata(&CLOSE_PUBKEY, &ME_PUBKEY, &TOKEN_PROGRAM_ID);

        [
            spl_token::instruction::transfer_checked(
                &TOKEN_PROGRAM_ID,
                wallet_token_ata,
                &ME_PUBKEY,
                &me_ata,
                wallet_pubkey,
                &[wallet_pubkey],
                rent,
                6u8,
            )
            .expect("Tip ix to be valid"),
            solana_sdk::system_instruction::transfer(
                payer_pubkey,
                JITO_BUNDLE_TIP_ACCOUNTS.choose(&mut rng).unwrap(),
                sol_to_lamports(jito_tip_amount),
            ),
        ]
    }
}
