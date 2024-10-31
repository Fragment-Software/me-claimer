use std::fs::File;

use itertools::{EitherOrBoth, Itertools};
use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::utils::files::read_file_lines;

use super::{
    account::Account,
    constants::{CEX_ADDRESSES_FILE_PATH, DB_FILE_PATH, MNEMONICS_FILE_PATH, PROXIES_FILE_PATH},
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

    pub async fn new() -> eyre::Result<Self> {
        let mnemonics = read_file_lines(MNEMONICS_FILE_PATH).await.unwrap();
        let proxies = read_file_lines(PROXIES_FILE_PATH).await.unwrap();
        let cex_addresses = read_file_lines(CEX_ADDRESSES_FILE_PATH).await.unwrap();

        if cex_addresses.len() != mnemonics.len() {
            return Err(eyre::eyre!(
                "Amount of CEX-addresses ({}) does not match the number of private keys ({})",
                cex_addresses.len(),
                mnemonics.len()
            ));
        }

        let mut data = Vec::with_capacity(mnemonics.len());

        for (entry, cex_address) in mnemonics
            .into_iter()
            .zip_longest(proxies.into_iter())
            .zip(cex_addresses.into_iter())
        {
            let either_or_both = entry;

            let (private_key, proxy) = match either_or_both {
                EitherOrBoth::Both(pk, proxy) => (pk, Some(proxy)),
                EitherOrBoth::Left(pk) => (pk, None),
                EitherOrBoth::Right(_) => {
                    return Err(eyre::eyre!(
                        "Amount of proxies is greater than the number of private keys"
                    ));
                }
            };

            let account = Account::new(&private_key, proxy, &cex_address);
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
