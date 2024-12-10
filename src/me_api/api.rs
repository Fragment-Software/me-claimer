use reqwest::{header::HeaderMap, Method, Proxy};

use crate::utils::fetch::{send_http_request, RequestParams};

use super::{constants::CLAIM_AIRDROP_RECEIPT, schemas::ClaimBatchResponse, typedefs::RootJson};

pub async fn get_receipts(
    claim_wallets: &[&str],
    cu_price: u64,
    headers: HeaderMap,
    proxy: Option<&Proxy>,
) -> eyre::Result<Vec<ClaimBatchResponse>> {
    let query_batch = (0..claim_wallets.len())
        .map(|_| "ixs.newClaimBatch")
        .collect::<Vec<&str>>()
        .join(",");

    let full_url = format!("{}{}", CLAIM_AIRDROP_RECEIPT, query_batch);

    let query = RootJson::to_string(
        claim_wallets,
        "tge-airdrop-final",
        "acAvyneD7adS3yrXUp41c1AuoYoYRhnjeAWH9stbdTf",
        false,
        cu_price,
    )
    .expect("Failed to stringify receipt query");

    let query_args = [("batch", "1"), ("input", query.as_str())]
        .into_iter()
        .collect();

    let request_params = RequestParams {
        url: &full_url,
        method: Method::GET,
        body: None::<serde_json::Value>,
        query_args: Some(query_args),
        proxy,
        headers: Some(headers),
    };

    let response_body = send_http_request::<Vec<ClaimBatchResponse>>(request_params).await?;

    Ok(response_body)
}
