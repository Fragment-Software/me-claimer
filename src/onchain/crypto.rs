use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer as SolSigner};
use std::num::ParseIntError;

use bip39::Mnemonic;
use ring::hmac::Key;
use sodiumoxide::crypto::sign::{ed25519, PublicKey, SecretKey, Seed};

use super::constants::{CURVE, PATH};

pub fn get_wallet(secret: &str) -> eyre::Result<Keypair> {
    match mnemonic_to_private_key(secret) {
        Ok(private_key) => Ok(Keypair::from_base58_string(&private_key)),
        Err(_) => {
            let decoded_key = solana_sdk::bs58::decode(secret)
                .into_vec()
                .map_err(|_| eyre::eyre!("Invalid private key: {secret}"))?;
            Ok(Keypair::from_bytes(&decoded_key)?)
        }
    }
}

pub fn get_address(signer: &Keypair) -> Pubkey {
    signer.pubkey()
}

#[derive(Debug)]
pub struct Keys {
    pub key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

fn mnemonic_to_seed(mnemonic: &str) -> eyre::Result<Vec<u8>> {
    let mnemonic = match Mnemonic::parse(mnemonic) {
        Ok(mnemo) => mnemo,
        Err(e) => eyre::bail!("{e}"),
    };

    let seed = mnemonic.to_seed("");
    Ok(seed.to_vec())
}

fn to_hex(bytes: Vec<u8>) -> String {
    hex::encode(bytes)
}

fn replace_derive(val: &str) -> String {
    val.replace("'", "")
}

fn get_master_key_from_seed(seed: &str, curve: &str) -> Keys {
    let seed_bytes = hex::decode(seed).expect("Invalid hex string");

    let key = Key::new(ring::hmac::HMAC_SHA512, curve.as_bytes());
    let tag = ring::hmac::sign(&key, &seed_bytes);

    let result = tag.as_ref();
    let (il, ir) = result.split_at(32);

    Keys {
        key: il.to_vec(),
        chain_code: ir.to_vec(),
    }
}

fn parse_segments(path: &str) -> Result<Vec<u32>, ParseIntError> {
    path.split('/')
        .skip(1)
        .map(replace_derive)
        .map(|el| el.parse::<u32>())
        .collect()
}

fn ckd_priv(keys: &Keys, index: u32) -> Keys {
    let index_buffer = index.to_be_bytes();

    let mut data = vec![0u8];
    data.extend_from_slice(&keys.key);
    data.extend_from_slice(&index_buffer);

    let key = Key::new(ring::hmac::HMAC_SHA512, &keys.chain_code);
    let tag = ring::hmac::sign(&key, &data);

    let result = tag.as_ref();
    let (il, ir) = result.split_at(32);

    Keys {
        key: il.to_vec(),
        chain_code: ir.to_vec(),
    }
}

fn derive_path(path: &str, seed: &str, curve: &str, offset: u32) -> eyre::Result<Keys> {
    let master_keys = get_master_key_from_seed(seed, curve);
    let segments = parse_segments(path).map_err(|e| e.to_string()).unwrap();

    let derived_keys = segments
        .into_iter()
        .fold(master_keys, |parent_keys, segment| {
            ckd_priv(&parent_keys, segment + offset)
        });

    Ok(derived_keys)
}

fn derive(mnemonic: &str, path: &str, curve: &str) -> eyre::Result<Vec<u8>> {
    let seed = mnemonic_to_seed(mnemonic)?;
    let hex_seed = to_hex(seed);

    let Keys { key, .. }: Keys = derive_path(path, &hex_seed, curve, 0x80000000)?;

    Ok(key)
}

fn mnemonic_to_private_key(mnemonic: &str) -> eyre::Result<String> {
    sodiumoxide::init().unwrap();
    let key = derive(mnemonic, PATH, CURVE)?;

    let seed: Seed = Seed::from_slice(&key).expect("Invalid private key");
    let (public_key, _): (PublicKey, SecretKey) = ed25519::keypair_from_seed(&seed);

    let mut secret_key: Vec<u8> = Vec::new();
    secret_key.extend_from_slice(&key);
    secret_key.extend_from_slice(public_key.as_ref());

    Ok(solana_sdk::bs58::encode(secret_key.clone()).into_string())
}
