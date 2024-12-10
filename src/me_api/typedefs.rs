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
    #[serde(flatten)]
    entries: std::collections::HashMap<String, OuterJson>,
}

impl RootJson {
    pub fn to_string(
        claim_wallets: &[&str],
        allocation_event: &str,
        ns: &str,
        enable_stake: bool,
        priority_fee_micro_lamports: u64,
    ) -> eyre::Result<String> {
        let entries: std::collections::HashMap<String, OuterJson> = claim_wallets
            .iter()
            .enumerate()
            .map(|(i, &wallet)| {
                (
                    i.to_string(),
                    OuterJson {
                        json: InnerJson {
                            claim_wallet: wallet.to_string(),
                            allocation_event: allocation_event.to_string(),
                            ns: ns.to_string(),
                            enable_stake,
                            priority_fee_micro_lamports,
                        },
                    },
                )
            })
            .collect();

        let query = RootJson { entries };
        Ok(serde_json::to_string(&query)?)
    }
}
