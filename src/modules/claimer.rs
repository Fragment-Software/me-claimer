use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use solana_sdk::{
    instruction::Instruction,
    native_token::{lamports_to_sol, sol_to_lamports},
    pubkey::Pubkey,
    signature::Signature,
    signer::Signer,
    transaction::{Transaction, VersionedTransaction},
};
use tokio::{sync::Mutex, task::JoinSet};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    jito::{jito_lib::JitoJsonRpcSDK, utils::check_final_bundle_status},
    onchain::{
        constants::{ME_PUBKEY, TOKEN_PROGRAM_ID},
        crypto::get_wallet,
        derive::derive_ata,
        ixs::Instructions,
        typedefs::CreateAtaArgs,
    },
};

use super::prepare_txs::get_claim_txs;

pub async fn claim_me(db: Arc<Mutex<Database>>, config: &Config) -> eyre::Result<()> {
    let config = Arc::new(config.clone());

    let unclaimed_wallets: Vec<Arc<Mutex<Account>>> = db
        .lock()
        .await
        .0
        .iter()
        .filter(|a| !a.get_claimed())
        .map(|a| Arc::new(Mutex::new(a.clone())))
        .collect();

    let mut join_set = JoinSet::new();

    let mut locked_wallets = Vec::new();

    for wallet in &unclaimed_wallets {
        let account = wallet.lock().await.clone();
        locked_wallets.push(account);
    }

    let txs = get_claim_txs(&locked_wallets, &config).await?;

    for (index, account) in unclaimed_wallets.into_iter().enumerate() {
        let should_process = {
            let account_locked = account.lock().await;
            !account_locked.get_claimed()
        };

        if should_process {
            let config_clone = Arc::clone(&config);
            let account_clone = Arc::clone(&account);
            let txs_for_account = txs[index].clone();
            let db_clone = Arc::clone(&db);

            join_set.spawn(process_account(
                account_clone,
                txs_for_account,
                config_clone,
                db_clone,
            ));

            if join_set.len() >= config.parallelism {
                if let Some(Err(e)) = join_set.join_next().await {
                    tracing::error!("Task failed: {}", e);
                }
            }
        }
    }

    while let Some(result) = join_set.join_next().await {
        if let Err(e) = result {
            tracing::error!("Task failed: {}", e);
        }
    }

    Ok(())
}

async fn get_ixs(
    allocation: u64,
    wallet_pubkey: &Pubkey,
    cex_pubkey: Option<&str>,
    payer_pubkey: &Pubkey,
    config: &Arc<Config>,
) -> eyre::Result<Vec<Instruction>> {
    let mut ixs = vec![];

    let (wallet_token_ata, _) = derive_ata(wallet_pubkey, &ME_PUBKEY, &TOKEN_PROGRAM_ID);

    let rent = sol_to_lamports(lamports_to_sol(allocation) * 0.05);

    let tip_ix = Instructions::tip_ix(
        &wallet_token_ata,
        wallet_pubkey,
        payer_pubkey,
        config.jito_tip_amount,
        rent,
    );

    ixs.extend_from_slice(&tip_ix);

    if config.withdraw_to_cex && cex_pubkey.is_some() {
        let cex_pubkey = Pubkey::from_str(cex_pubkey.unwrap())?;

        let (cex_token_ata, _) = derive_ata(&cex_pubkey, &ME_PUBKEY, &TOKEN_PROGRAM_ID);

        let create_ata_args = CreateAtaArgs {
            funding_address: *payer_pubkey,
            associated_account_address: cex_token_ata,
            wallet_address: cex_pubkey,
            token_mint_address: ME_PUBKEY,
            token_program_id: TOKEN_PROGRAM_ID,
            instruction: 1,
        };

        ixs.push(Instructions::create_ata(create_ata_args));

        let transfer_ix = spl_token::instruction::transfer_checked(
            &TOKEN_PROGRAM_ID,
            &wallet_token_ata,
            &ME_PUBKEY,
            &cex_token_ata,
            wallet_pubkey,
            &[wallet_pubkey],
            allocation - rent,
            6u8,
        )?;

        ixs.push(transfer_ix);
    }

    Ok(ixs)
}

async fn process_account(
    account: Arc<Mutex<Account>>,
    txs: Vec<HashMap<std::string::String, u64>>,
    config: Arc<Config>,
    db: Arc<Mutex<Database>>,
) -> eyre::Result<()> {
    let jito_provider = JitoJsonRpcSDK::new(
        "https://mainnet.block-engine.jito.wtf/api/v1",
        None,
        &account.lock().await.proxy(),
    );

    let wallet = account.lock().await.keypair()?;

    let payer_kp = match config.use_external_fee_pay {
        true => get_wallet(&config.external_fee_payer_secret)?,
        false => wallet.insecure_clone(),
    };

    let signing_keypairs = match config.use_external_fee_pay {
        true => vec![&payer_kp, &wallet],
        false => vec![&wallet],
    };

    for tx_map in txs {
        for (tx_base58, allocation) in tx_map.iter() {
            let tx_bytes = solana_sdk::bs58::decode(tx_base58).into_vec()?;
            let mut claim_tx = bincode::deserialize::<VersionedTransaction>(&tx_bytes)?;

            let new_signature: Signature = wallet.sign_message(&claim_tx.message.serialize());
            claim_tx.signatures[0] = new_signature;

            let serialized_claim_tx =
                solana_sdk::bs58::encode(bincode::serialize(&claim_tx)?).into_string();

            let instructions = get_ixs(
                *allocation,
                &wallet.pubkey(),
                account.lock().await.get_cex_address(),
                &payer_kp.pubkey(),
                &config,
            )
            .await?;

            let recent_blockhash = claim_tx.message.recent_blockhash();

            let inner_tx = Transaction::new_signed_with_payer(
                &instructions,
                Some(&payer_kp.pubkey()),
                &signing_keypairs,
                *recent_blockhash,
            );

            let serialized_inner_tx =
                solana_sdk::bs58::encode(bincode::serialize(&inner_tx)?).into_string();

            let bundle = serde_json::json!([serialized_claim_tx, serialized_inner_tx]);

            let uuid = None;

            let response = jito_provider.send_bundle(Some(bundle), uuid).await?;

            let bundle_uuid = response["result"]
                .as_str()
                .ok_or_else(|| eyre::eyre!("Failed to get bundle UUID from response"))?;

            let max_retries = 10;
            let retry_delay = Duration::from_secs(5);

            for attempt in 1..=max_retries {
                let status_response = jito_provider
                    .get_in_flight_bundle_statuses(vec![bundle_uuid.to_string()])
                    .await?;

                if let Some(result) = status_response.get("result") {
                    if let Some(value) = result.get("value") {
                        if let Some(statuses) = value.as_array() {
                            if let Some(bundle_status) = statuses.first() {
                                if let Some(status) = bundle_status.get("status") {
                                    match status.as_str() {
                                        Some("Landed") => {
                                            return check_final_bundle_status(
                                                &jito_provider,
                                                bundle_uuid,
                                                account,
                                                db,
                                            )
                                            .await;
                                        }
                                        Some("Pending") => {}
                                        Some(_) => {}
                                        None => {}
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(error) = status_response.get("error") {
                    tracing::error!(
                        "{}: Error checking bundle status: {:?}",
                        wallet.pubkey(),
                        error
                    );
                }

                if attempt < max_retries {
                    tokio::time::sleep(retry_delay).await;
                }
            }
        }
    }

    Ok(())
}
