use std::time::Duration;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Signature, signer::Signer,
    transaction::VersionedTransaction,
};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    jup_api::api::{quote, swap},
    onchain::tx::send_and_confirm_tx,
    utils::misc::{pretty_sleep, swap_ip_address},
};

pub async fn swap_testme(mut db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    while let Some(account) = db.get_random_account_with_filter(|a| !a.get_swapped()) {
        if let Err(e) = process_account(&provider, account, config).await {
            tracing::error!("{}", e);
        } else {
            account.set_swapped(true);
            db.update();
        };

        pretty_sleep(config.claim_sleep_range).await;
    }

    Ok(())
}

async fn process_account(
    provider: &RpcClient,
    account: &mut Account,
    config: &Config,
) -> eyre::Result<()> {
    let wallet = account.keypair();
    let wallet_pubkey = account.get_pubkey();
    let proxy = account.proxy();

    tracing::info!("Wallet address: `{}`", wallet.pubkey());

    if config.mobile_proxies {
        tracing::info!("Changing IP address");
        swap_ip_address(&config.swap_ip_link).await?;
    }

    let quote_response = quote(proxy.as_ref()).await?;
    let swap_resp = swap(&wallet_pubkey, quote_response, proxy.as_ref()).await?;

    let tx_base58 = swap_resp.swap_transaction;

    let tx_bytes = solana_sdk::bs58::decode(tx_base58).into_vec()?;

    let mut tx = bincode::deserialize::<VersionedTransaction>(&tx_bytes)?;

    let new_signature: Signature = wallet.sign_message(&tx.message.serialize());

    tx.signatures[0] = new_signature;

    send_and_confirm_tx(provider, tx).await?;

    Ok(())
}
