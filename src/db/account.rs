#![allow(dead_code)]
#![allow(unused_variables)]

use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

use crate::onchain::crypto::{get_address, get_wallet};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Account {
    secret: String,
    cex_address: Option<String>,
    proxy: String,
    closed_ata: bool,
    collected_sol: bool,
}

impl Account {
    pub fn new(secret: &str, cex_address: Option<String>, proxy: &str) -> Self {
        Self {
            secret: secret.to_string(),
            cex_address,
            proxy: proxy.to_string(),
            ..Default::default()
        }
    }

    pub fn proxy(&self) -> Proxy {
        Proxy::all(&self.proxy).expect("Proxy to be valid")
    }

    pub fn keypair(&self) -> eyre::Result<Keypair> {
        get_wallet(&self.secret)
    }

    pub fn get_pubkey(&self) -> Pubkey {
        let wallet = self.keypair().expect("Failed to get keypair");
        get_address(&wallet)
    }

    pub fn get_cex_address(&self) -> Option<&str> {
        self.cex_address.as_deref()
    }

    pub fn get_closed_ata(&self) -> bool {
        self.closed_ata
    }

    pub fn set_closed_ata(&mut self, closed: bool) {
        self.closed_ata = closed
    }

    pub fn get_collected_sol(&self) -> bool {
        self.collected_sol
    }

    pub fn set_collected_sol(&mut self, collected_sol: bool) {
        self.collected_sol = collected_sol
    }
}
