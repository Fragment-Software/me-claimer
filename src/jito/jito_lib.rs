use reqwest::{Client, Proxy};
use serde_json::{json, Value};
use std::fmt;

pub struct JitoJsonRpcSDK {
    base_url: String,
    uuid: Option<String>,
    client: Client,
}

#[derive(Debug)]
pub struct PrettyJsonValue(pub Value);

impl fmt::Display for PrettyJsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self.0).unwrap())
    }
}

impl From<Value> for PrettyJsonValue {
    fn from(value: Value) -> Self {
        PrettyJsonValue(value)
    }
}

impl JitoJsonRpcSDK {
    pub fn new(base_url: &str, uuid: Option<String>, proxy: &Proxy) -> Self {
        let client = Client::builder()
            .proxy(proxy.clone())
            .build()
            .unwrap_or_else(|err| {
                tracing::error!("Failed to build a client with proxy: {proxy:?}. Error: {err}");
                Client::new()
            });

        Self {
            base_url: base_url.to_string(),
            uuid,
            client,
        }
    }

    async fn send_request(
        &self,
        endpoint: &str,
        method: &str,
        params: Option<Value>,
    ) -> eyre::Result<Value, reqwest::Error> {
        let url = format!("{}{}", self.base_url, endpoint);

        let data = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params.unwrap_or(json!([]))
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&data)
            .send()
            .await?;

        let body = response.json::<Value>().await?;

        Ok(body)
    }

    pub async fn get_bundle_statuses(&self, bundle_uuids: Vec<String>) -> eyre::Result<Value> {
        let endpoint = if let Some(uuid) = &self.uuid {
            format!("/bundles?uuid={}", uuid)
        } else {
            "/bundles".to_string()
        };

        // Construct the params as a list within a list
        let params = json!([bundle_uuids]);

        self.send_request(&endpoint, "getBundleStatuses", Some(params))
            .await
            .map_err(|e| eyre::eyre!("Request error: {}", e))
    }

    pub async fn send_bundle(
        &self,
        params: Option<Value>,
        uuid: Option<&str>,
    ) -> eyre::Result<Value> {
        let mut endpoint = "/bundles".to_string();

        if let Some(uuid) = uuid {
            endpoint = format!("{}?uuid={}", endpoint, uuid);
        }

        // Ensure params is an array of transactions
        let transactions = match params {
            Some(Value::Array(transactions)) => {
                if transactions.is_empty() {
                    eyre::bail!("Bundle must contain at least one transaction");
                }
                if transactions.len() > 5 {
                    eyre::bail!("Bundle can contain at most 5 transactions");
                }
                transactions
            }
            _ => {
                eyre::bail!("Invalid bundle format: expected an array of transactions")
            }
        };

        // Wrap the transactions array in another array
        let params = json!([transactions]);

        // Send the wrapped transactions array
        self.send_request(&endpoint, "sendBundle", Some(params))
            .await
            .map_err(|e| eyre::eyre!("Request error: {}", e))
    }

    pub async fn get_in_flight_bundle_statuses(
        &self,
        bundle_uuids: Vec<String>,
    ) -> eyre::Result<Value> {
        let endpoint = if let Some(uuid) = &self.uuid {
            format!("/bundles?uuid={}", uuid)
        } else {
            "/bundles".to_string()
        };

        // Construct the params as a list within a list
        let params = json!([bundle_uuids]);

        self.send_request(&endpoint, "getInflightBundleStatuses", Some(params))
            .await
            .map_err(|e| eyre::eyre!("Request error: {}", e))
    }
}
