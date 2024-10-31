use std::time::Duration;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Signature, signer::Signer,
    transaction::VersionedTransaction,
};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    me_api::api::get_receipt,
    onchain::tx::send_and_confirm_tx,
    utils::misc::{pretty_sleep, swap_ip_address},
};

pub async fn claim_testme(mut db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    while let Some(account) = db.get_random_account_with_filter(|a| !a.get_claimed()) {
        if let Err(e) = process_account(&provider, account, config).await {
            tracing::error!("{}", e);
        } else {
            account.set_claimed(true);
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

    let receipt = get_receipt(&wallet_pubkey.to_string(), proxy.as_ref()).await?;

    let response = receipt.first().unwrap();

    if let Some(err) = &response.error {
        tracing::warn!("{} {}", err.json.code, err.json.message);
        return Ok(());
    }

    let tx_base58 = response
        .result
        .as_ref()
        .unwrap()
        .data
        .json
        .first()
        .unwrap()
        .tx_base58
        .clone();

    let tx_bytes = solana_sdk::bs58::decode(tx_base58).into_vec()?;

    let mut tx = bincode::deserialize::<VersionedTransaction>(&tx_bytes)?;

    let new_signature: Signature = wallet.sign_message(&tx.message.serialize());

    tx.signatures[0] = new_signature;

    send_and_confirm_tx(provider, tx).await?;

    Ok(())
}
