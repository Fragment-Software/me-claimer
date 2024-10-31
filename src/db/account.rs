use std::str::FromStr;

use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

use crate::onchain::crypto::mnemonic_to_private_key;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Account {
    mnemonic: String,
    private_key: String,
    proxy: Option<String>,
    address: String,
    cex_address: String,
    claimed: bool,
    swapped: bool,
    transfered: bool,
}

impl Account {
    pub fn new(mnemonic: &str, proxy: Option<String>, cex_address: &str) -> Self {
        let private_key = mnemonic_to_private_key(mnemonic);

        let signer = Keypair::from_base58_string(&private_key);
        let address = signer.pubkey();

        Self {
            mnemonic: mnemonic.to_string(),
            private_key,
            proxy,
            address: address.to_string(),
            cex_address: cex_address.to_string(),
            ..Default::default()
        }
    }

    pub fn proxy(&self) -> Option<Proxy> {
        self.proxy
            .as_ref()
            .map(|proxy| Proxy::all(proxy).expect("Proxy to be valid"))
    }

    pub fn keypair(&self) -> Keypair {
        Keypair::from_base58_string(&self.private_key)
    }

    pub fn get_pubkey(&self) -> Pubkey {
        Pubkey::from_str(&self.address).expect("Address to be valid")
    }

    pub fn get_cex_address(&self) -> &str {
        &self.cex_address
    }

    pub fn set_claimed(&mut self, claimed: bool) {
        self.claimed = claimed
    }

    pub fn get_claimed(&self) -> bool {
        self.claimed
    }

    pub fn set_swapped(&mut self, swapped: bool) {
        self.swapped = swapped
    }

    pub fn get_swapped(&self) -> bool {
        self.swapped
    }

    pub fn set_transfered(&mut self, transfered: bool) {
        self.transfered = transfered
    }

    pub fn get_transfered(&self) -> bool {
        self.transfered
    }
}
