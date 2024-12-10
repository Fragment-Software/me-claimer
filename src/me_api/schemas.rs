use serde::Deserialize;

#[derive(Deserialize)]
pub struct CosignerDistribution {
    #[serde(rename = "tokenAmount")]
    pub token_amount: u64,
}

#[derive(Deserialize)]
pub struct Metadata {
    #[serde(rename = "cosignerDistribution")]
    pub cosigner_distribution: CosignerDistribution,
}

#[derive(Deserialize)]
pub struct ClaimJson {
    pub metadata: Vec<Metadata>,
    #[serde(rename = "txBase58")]
    pub tx_base58: String,
}

#[derive(Deserialize)]
pub struct JsonData<T> {
    pub json: Vec<T>,
}

#[derive(Deserialize)]
pub struct ClaimResult<T> {
    pub data: JsonData<T>,
}

#[derive(Deserialize)]
pub struct ClaimBatchResponse<T> {
    pub result: Option<ClaimResult<T>>,
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
