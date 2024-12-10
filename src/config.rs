use serde::Deserialize;
use std::path::Path;

#[allow(unused)]
const CONFIG_FILE_PATH: &str = "data/config.toml";

#[derive(Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Config {
    pub solana_rpc_url: String,
    pub me_proxy_url: String,
    pub parallelism: usize,
    pub collect_me: bool,
    pub collect_sol: bool,
    pub withdraw_to_cex: bool,
    pub jito_tip_amount: f64,
    pub cu_price: u64,
    pub claim_sleep_range: [u64; 2],
    pub use_external_fee_pay: bool,
    pub external_fee_payer_secret: String,
}

impl Config {
    async fn read_from_file(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let cfg_str = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&cfg_str)?)
    }

    pub async fn read_default() -> Self {
        Self::read_from_file(CONFIG_FILE_PATH)
            .await
            .expect("Default config to be valid")
    }
}
