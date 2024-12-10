use std::sync::LazyLock;

use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const ME_PUBKEY: Pubkey = pubkey!("MEFNBXixkEbait3xn9bkm8WsJzXtVsaJEn4c8Sam21u");

pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub static CLOSE_PUBKEY: LazyLock<Pubkey> = LazyLock::new(|| {
    Pubkey::new_from_array([
        156, 194, 179, 36, 147, 69, 5, 154, 187, 52, 104, 42, 42, 191, 111, 230, 84, 196, 250, 181,
        2, 30, 228, 228, 148, 3, 135, 142, 129, 86, 251, 163,
    ])
});

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

pub const PATH: &str = "m/44'/501'/0'/0'";

pub const CURVE: &str = "ed25519 seed";
