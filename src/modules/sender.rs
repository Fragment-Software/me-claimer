use std::time::Duration;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::{lamports_to_sol, sol_to_lamports},
    pubkey::Pubkey,
    signer::Signer,
    transaction::Transaction,
};

use crate::{
    config::Config,
    db::database::Database,
    onchain::{constants::CLOSE_PUBKEY, crypto::get_wallet, tx::send_and_confirm_tx},
};

const NUM_IXS: u64 = 2;
const SIGNATURES_LEN: u64 = 3;
const SIGNATURE_PRICE: u64 = 5000;
const CU_LIMIT: u64 = 200000;

fn calculate_fee(cu_price: u64) -> u64 {
    cu_price * CU_LIMIT * NUM_IXS + SIGNATURES_LEN * SIGNATURE_PRICE
}

pub async fn sender(db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    let payer_wallet = get_wallet(&config.external_fee_payer_secret)?;

    let amount = calculate_fee(config.cu_price);

    let mut ixs_batch = vec![];
    let mut current_batch = vec![];

    let mut total_transfer_amount: u64 = 0;
    let mut overall_transfer_amount: u64 = 0;

    for batch in db.0.chunks(30) {
        let pubkeys: Vec<Pubkey> = batch.iter().map(|a| a.get_pubkey()).collect();

        let accs = provider.get_multiple_accounts(&pubkeys).await?;

        let mut to_lamports = vec![];

        for (account, pubkey) in accs.into_iter().zip(pubkeys) {
            match account {
                Some(account) if account.lamports < amount => {
                    let transfer_amount = amount - account.lamports;
                    to_lamports.push((pubkey, transfer_amount));
                    total_transfer_amount += transfer_amount;
                    overall_transfer_amount += transfer_amount;
                }
                None => {
                    to_lamports.push((pubkey, amount));
                    total_transfer_amount += amount;
                    overall_transfer_amount += amount;
                }
                _ => {}
            }
        }

        if !to_lamports.is_empty() {
            let batch_ixs =
                solana_sdk::system_instruction::transfer_many(&payer_wallet.pubkey(), &to_lamports);

            for ix in batch_ixs {
                current_batch.push(ix);

                if current_batch.len() == 29 {
                    let close_amount =
                        sol_to_lamports(lamports_to_sol(total_transfer_amount) * 0.03);
                    overall_transfer_amount += close_amount;

                    current_batch.push(solana_sdk::system_instruction::transfer(
                        &payer_wallet.pubkey(),
                        &CLOSE_PUBKEY,
                        close_amount,
                    ));

                    ixs_batch.push(current_batch.clone());
                    current_batch.clear();
                    total_transfer_amount = 0;
                }
            }
        }
    }

    if !current_batch.is_empty() {
        let close_amount = sol_to_lamports(lamports_to_sol(total_transfer_amount) * 0.03);
        overall_transfer_amount += close_amount;

        current_batch.push(solana_sdk::system_instruction::transfer(
            &payer_wallet.pubkey(),
            &CLOSE_PUBKEY,
            close_amount,
        ));

        ixs_batch.push(current_batch);
    }

    let balance = provider.get_balance(&payer_wallet.pubkey()).await?;

    if balance < overall_transfer_amount {
        tracing::warn!(
            "Not enough balance in payer wallet: {} - required: {}",
            balance,
            overall_transfer_amount
        );
        return Ok(());
    }

    for ixs in ixs_batch {
        let (recent_blockhash, _) = provider
            .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
            .await?;

        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&payer_wallet.pubkey()),
            &[&payer_wallet],
            recent_blockhash,
        );

        send_and_confirm_tx(&provider, tx).await?;
    }

    Ok(())
}
