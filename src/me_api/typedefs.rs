use serde::Serialize;

#[derive(Serialize)]
pub struct InnerJson {
    #[serde(rename = "claimWallet")]
    claim_wallet: String,
    #[serde(rename = "allocationEvent")]
    allocation_event: String,
    ns: String,
    #[serde(rename = "enableStake")]
    enable_stake: bool,
    #[serde(rename = "priorityFeeMicroLamports")]
    priority_fee_micro_lamports: u64,
}

#[derive(Serialize)]
pub struct OuterJson {
    json: InnerJson,
}

#[derive(Serialize)]
pub struct RootJson {
    #[serde(rename = "0")]
    inner: OuterJson,
}

impl RootJson {
    pub fn to_string(
        claim_wallet: &str,
        allocation_event: &str,
        ns: &str,
        enable_stake: bool,
        priority_fee_micro_lamports: u64,
    ) -> eyre::Result<String, serde_json::Error> {
        let query = Self {
            inner: OuterJson {
                json: InnerJson {
                    claim_wallet: claim_wallet.to_string(),
                    allocation_event: allocation_event.to_string(),
                    ns: ns.to_string(),
                    enable_stake,
                    priority_fee_micro_lamports,
                },
            },
        };

        serde_json::to_string(&query)
    }
}
