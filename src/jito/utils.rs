use std::sync::Arc;

use tokio::sync::Mutex;

use crate::db::{account::Account, database::Database};

use super::jito_lib::JitoJsonRpcSDK;

#[derive(Debug)]
pub struct BundleStatus {
    confirmation_status: Option<String>,
    err: Option<serde_json::Value>,
    transactions: Option<Vec<String>>,
}

pub async fn check_final_bundle_status(
    jito_sdk: &JitoJsonRpcSDK,
    bundle_uuid: &str,
    account: Arc<Mutex<Account>>,
    db: Arc<Mutex<Database>>,
) -> eyre::Result<()> {
    let max_retries = 10;
    let retry_delay = tokio::time::Duration::from_secs(5);

    for attempt in 1..=max_retries {
        let status_response = jito_sdk
            .get_bundle_statuses(vec![bundle_uuid.to_string()])
            .await?;
        let bundle_status = get_bundle_status(&status_response)?;

        match bundle_status.confirmation_status.as_deref() {
            Some("confirmed") => {
                check_transaction_error(&bundle_status)?;
            }
            Some("finalized") => {
                check_transaction_error(&bundle_status)?;
                account.lock().await.set_claimed(true);
                db.lock().await.update();
                print_transaction_url(&bundle_status);
                return Ok(());
            }
            Some(_) => {}
            None => {}
        }

        if attempt < max_retries {
            tokio::time::sleep(retry_delay).await;
        }
    }

    eyre::bail!(
        "Failed to get finalized status after {} attempts",
        max_retries
    )
}

pub fn get_bundle_status(status_response: &serde_json::Value) -> eyre::Result<BundleStatus> {
    status_response
        .get("result")
        .and_then(|result| result.get("value"))
        .and_then(|value| value.as_array())
        .and_then(|statuses| statuses.first())
        .ok_or_else(|| eyre::eyre!("Failed to parse bundle status"))
        .map(|bundle_status| BundleStatus {
            confirmation_status: bundle_status
                .get("confirmation_status")
                .and_then(|s| s.as_str())
                .map(String::from),
            err: bundle_status.get("err").cloned(),
            transactions: bundle_status
                .get("transactions")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                }),
        })
}

pub fn check_transaction_error(bundle_status: &BundleStatus) -> eyre::Result<()> {
    if let Some(err) = &bundle_status.err {
        if err["Ok"].is_null() {
            tracing::error!("Transaction executed without errors.");
            Ok(())
        } else {
            tracing::error!("Transaction encountered an error: {:?}", err);
            eyre::bail!("Transaction encountered an error")
        }
    } else {
        Ok(())
    }
}

pub fn print_transaction_url(bundle_status: &BundleStatus) {
    if let Some(transactions) = &bundle_status.transactions {
        if let Some(tx_id) = transactions.first() {
            tracing::info!("Transaction URL: https://solscan.io/tx/{}", tx_id);
        } else {
            tracing::warn!("Unable to extract transaction ID.");
        }
    } else {
        tracing::warn!("No transactions found in the bundle status.");
    }
}
