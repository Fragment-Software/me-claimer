use reqwest::{Method, Proxy};

use crate::utils::fetch::{send_http_request, RequestParams};

use super::{
    constants::CLAIM_AIRDROP_RECEIPT,
    schemas::{ClaimBatchResponse, ClaimJson},
    typedefs::RootJson,
};

pub async fn get_receipt(
    wallet_address: &str,
    proxy: Option<&Proxy>,
) -> eyre::Result<Vec<ClaimBatchResponse<ClaimJson>>> {
    let query = RootJson::to_string(
        wallet_address,
        "testme-airdrop-0",
        "5PoydnnQyvsuH9YoLH1gDP3Sfn9HrDwtrBjpzkaHEKP1",
        false,
        80000,
    )
    .expect("Failed to stringify receipt query");

    let query_args = [("batch", "1"), ("input", query.as_str())]
        .into_iter()
        .collect();

    let request_params = RequestParams {
        url: CLAIM_AIRDROP_RECEIPT,
        method: Method::GET,
        body: None::<serde_json::Value>,
        query_args: Some(query_args),
        proxy,
        headers: None,
    };

    let response_body =
        send_http_request::<Vec<ClaimBatchResponse<ClaimJson>>>(request_params).await?;

    Ok(response_body)
}
