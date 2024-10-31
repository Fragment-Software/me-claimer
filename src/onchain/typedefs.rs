use solana_sdk::pubkey::Pubkey;

pub struct CreateAtaArgs {
    pub funding_address: Pubkey,
    pub associated_account_address: Pubkey,
    pub wallet_address: Pubkey,
    pub token_mint_address: Pubkey,
    pub token_program_id: Pubkey,
    pub instruction: u8,
}
