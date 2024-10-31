use solana_program::instruction::{AccountMeta, Instruction};

use super::constants::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID};
use super::typedefs::CreateAtaArgs;

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
