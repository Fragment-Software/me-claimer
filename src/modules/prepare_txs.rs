use std::collections::HashMap;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Proxy,
};

use crate::{
    config::Config,
    db::{account::Account, constants::HEADERS_FILE_PATH},
    me_api::api::get_receipts,
    utils::files::read_json_to_map,
};

type ClaimTxs = eyre::Result<Vec<Vec<HashMap<String, u64>>>>;

pub async fn get_claim_txs(accounts: &mut [Account], config: &Config) -> ClaimTxs {
    let mut headers = HeaderMap::new();

    let headers_map = read_json_to_map(HEADERS_FILE_PATH).await?;

    for (k, v) in headers_map.into_iter() {
        headers.insert(k.parse::<HeaderName>()?, HeaderValue::from_str(&v)?);
    }

    let claim_wallets: Vec<String> = accounts
        .iter()
        .map(|a| a.get_pubkey().to_string())
        .collect();
    let claim_wallets_refs: Vec<&str> = claim_wallets.iter().map(String::as_str).collect();

    let proxy = Proxy::all(&config.me_proxy_url).expect("Invalid proxy URL");

    let mut txns: Vec<Vec<HashMap<String, u64>>> = vec![Vec::new(); accounts.len()];

    for (batch_index, batch) in claim_wallets_refs.chunks(40).enumerate() {
        match get_receipts(batch, config.cu_price, headers.clone(), Some(&proxy)).await {
            Ok(receipts) => {
                for (index, receipt) in receipts.into_iter().enumerate() {
                    let global_index = batch_index * 40 + index;
                    if let Some(err) = &receipt.error {
                        if err.json.code == -32600 && err.json.message == "No instructions to fetch"
                        {
                            tracing::info!("{}: Already claimed", batch[batch_index]);
                        } else {
                            tracing::warn!("Error {}: {}", err.json.code, err.json.message);
                        }
                    } else if let Some(result) = receipt.result {
                        for (index, json) in result.data.json.transactions.into_iter().enumerate() {
                            let token_amount = if let Some(distibution) =
                                json.metadata[index].merkle_distribution.as_ref()
                            {
                                distibution.token_amount
                            } else if let Some(distibution) =
                                json.metadata[index].cosigner_distribution.as_ref()
                            {
                                distibution.token_amount
                            } else {
                                tracing::warn!(
                                    "Failed to get distribution type: {:#?}",
                                    json.metadata[index]
                                );
                                continue;
                            };

                            let tx_base58 = json.tx_base58.clone();

                            if global_index < txns.len() {
                                let mut tx_map = HashMap::new();
                                tx_map.insert(tx_base58, token_amount);
                                txns[global_index].push(tx_map);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Request failed: {}.", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    Ok(txns)
}
