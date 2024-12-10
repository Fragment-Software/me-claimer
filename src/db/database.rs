use std::fs::File;

use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::{config::Config, utils::files::read_file_lines};

use super::{
    account::Account,
    constants::{CEX_ADDRESSES_FILE_PATH, DB_FILE_PATH, PROXIES_FILE_PATH, SECRETS_FILE_PATH},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Database(pub Vec<Account>);

impl Database {
    async fn read_from_file(file_path: &str) -> eyre::Result<Self> {
        let contents = tokio::fs::read_to_string(file_path).await?;
        let db = serde_json::from_str::<Self>(&contents)?;
        Ok(db)
    }

    #[allow(unused)]
    pub async fn read() -> Self {
        Self::read_from_file(DB_FILE_PATH)
            .await
            .expect("Default db to be valid")
    }

    pub async fn new(config: &Config) -> eyre::Result<Self> {
        let secrets = read_file_lines(SECRETS_FILE_PATH).await.unwrap();

        let cex_addresses = if config.withdraw_to_cex || config.collect_me || config.collect_sol {
            let addresses = read_file_lines(CEX_ADDRESSES_FILE_PATH).await.unwrap();

            if addresses.len() != secrets.len() {
                return Err(eyre::eyre!(
                    "Amount of CEX-addresses ({}) does not match the number of secrets ({})",
                    addresses.len(),
                    secrets.len()
                ));
            }

            Some(addresses)
        } else {
            None
        };

        let proxies = read_file_lines(PROXIES_FILE_PATH).await.unwrap();

        if proxies.len() != secrets.len() {
            return Err(eyre::eyre!(
                "Amount of proxies ({}) does not match the number of secrets ({})",
                proxies.len(),
                secrets.len()
            ));
        }

        let unique_proxies: std::collections::HashSet<_> = proxies.iter().collect();
        if unique_proxies.len() != proxies.len() {
            return Err(eyre::eyre!(
                "Duplicate proxies detected in the file. Ensure all proxies are unique."
            ));
        }

        let mut data = Vec::with_capacity(secrets.len());

        for (i, secret) in secrets.into_iter().enumerate() {
            let cex_address = cex_addresses
                .as_ref()
                .and_then(|addresses| addresses.get(i).cloned());

            let proxy = &proxies[i];

            let account = Account::new(&secret, cex_address, proxy);
            data.push(account);
        }

        let db_file = File::create(DB_FILE_PATH)?;
        serde_json::to_writer_pretty(db_file, &data)?;

        Ok(Self(data))
    }

    pub fn get_random_account_with_filter<F>(&mut self, filter: F) -> Option<&mut Account>
    where
        F: Fn(&Account) -> bool,
    {
        let mut rng = thread_rng();

        self.0
            .iter_mut()
            .filter(|account| filter(account))
            .choose(&mut rng)
    }

    pub fn update(&self) {
        let file = File::create(DB_FILE_PATH).expect("Default database must be vaild");
        let _ = serde_json::to_writer_pretty(file, &self);
    }
}
