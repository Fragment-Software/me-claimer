use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::RpcClientConfig};
use solana_rpc_client::http_sender::HttpSender;
use solana_sdk::commitment_config::CommitmentConfig;
use std::{sync::Arc, time::Duration};

pub fn init_solana_rpc_client_with_retries(
    rpc_url: &str,
    http_client: reqwest::Client,
) -> Arc<RpcClient> {
    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
        .build_with_max_retries(5);

    let client_with_middleware = ClientBuilder::new(http_client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let sender = HttpSender::new_with_client_with_middleware(rpc_url, client_with_middleware);

    let client = RpcClient::new_sender(
        sender,
        RpcClientConfig::with_commitment(CommitmentConfig::processed()),
    );

    Arc::new(client)
}
