use reqwest::{Method, Proxy};
use solana_sdk::pubkey::Pubkey;

use crate::utils::fetch::{send_http_request, RequestParams};

use super::{
    constants::{QUOTE, SWAP, TESTME, USDC},
    schemas::{QuoteResponse, SwapBody, SwapResponse},
};

pub async fn quote(proxy: Option<&Proxy>) -> eyre::Result<QuoteResponse> {
    let query_args = [
        ("inputMint", TESTME),
        ("outputMint", USDC),
        ("amount", "987000000"),
        ("slippageBps", "50"),
        ("swapMode", "ExactIn"),
        ("onlyDirectRoutes", "false"),
        ("asLegacyTransaction", "false"),
        ("maxAccounts", "64"),
        ("minimizeSlippage", "false"),
    ]
    .into_iter()
    .collect();

    let request_params = RequestParams {
        url: QUOTE,
        method: Method::GET,
        body: None::<serde_json::Value>,
        query_args: Some(query_args),
        proxy,
        headers: None,
    };

    let response_body = send_http_request::<QuoteResponse>(request_params).await?;

    Ok(response_body)
}

pub async fn swap(
    wallet_pubkey: &Pubkey,
    quote_response: QuoteResponse,
    proxy: Option<&Proxy>,
) -> eyre::Result<SwapResponse> {
    let body = SwapBody::new(wallet_pubkey, quote_response);

    let request_params = RequestParams {
        url: SWAP,
        method: Method::POST,
        body: Some(body),
        query_args: None,
        proxy,
        headers: None,
    };

    let response_body = send_http_request::<SwapResponse>(request_params).await?;

    Ok(response_body)
}
