use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MerkleDistribution {
    #[serde(rename = "tokenAmount")]
    pub token_amount: u64,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    #[serde(rename = "merkleDistribution")]
    pub merkle_distribution: MerkleDistribution,
}

#[derive(Deserialize, Debug)]
pub struct ClaimJson {
    pub metadata: Vec<Metadata>,
    #[serde(rename = "txBase58")]
    pub tx_base58: String,
}

#[derive(Deserialize, Debug)]
pub struct Transactions {
    pub transactions: Vec<ClaimJson>,
}

#[derive(Deserialize, Debug)]
pub struct JsonData {
    pub json: Transactions,
}

#[derive(Deserialize, Debug)]
pub struct ClaimResult {
    pub data: JsonData,
}

#[derive(Deserialize)]
pub struct ClaimBatchResponse {
    pub result: Option<ClaimResult>,
    pub error: Option<ErrorReport>,
}

#[derive(Deserialize)]
pub struct ErrorReport {
    pub json: ErrorDescription,
}

#[derive(Deserialize)]
pub struct ErrorDescription {
    pub code: i64,
    pub message: String,
}
