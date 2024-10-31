use std::{str::FromStr, time::Duration};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, program_pack::Pack,
    pubkey::Pubkey, signer::Signer, transaction::Transaction,
};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    onchain::{
        constants::{TESTME_PUBKEY, TOKEN_PROGRAM_ID},
        derive::derive_ata,
        tx::send_and_confirm_tx,
    },
    utils::misc::pretty_sleep,
};

pub async fn collect_and_close(mut db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    while let Some(account) = db.get_random_account_with_filter(|a| !a.get_transfered()) {
        if let Err(e) = process_account(&provider, account).await {
            tracing::error!("{}", e);
        } else {
            account.set_transfered(true);
            db.update();
        };

        pretty_sleep(config.claim_sleep_range).await;
    }

    Ok(())
}

async fn get_ixs(
    provider: &RpcClient,
    wallet_pubkey: &Pubkey,
    cex_pubkey: &Pubkey,
) -> eyre::Result<Option<Vec<Instruction>>> {
    let mut ixs = vec![];

    let mut balance = provider.get_balance(wallet_pubkey).await?;

    let rent = provider
        .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
        .await?;

    let (wallet_token_ata, _) = derive_ata(wallet_pubkey, &TESTME_PUBKEY, &TOKEN_PROGRAM_ID);
    let token_ata_exist = provider.get_account_data(&wallet_token_ata).await.is_ok();

    let token_account = provider
        .get_token_account_balance(&wallet_token_ata)
        .await?;

    let token_account_balance = token_account.amount.parse::<u64>()?;

    if token_account_balance != 0 {
        tracing::warn!("TestME balance is not 0. Run Swap Module first");
        return Ok(None);
    }

    if token_ata_exist {
        ixs.push(
            spl_token::instruction::close_account(
                &TOKEN_PROGRAM_ID,
                &wallet_token_ata,
                wallet_pubkey,
                wallet_pubkey,
                &[wallet_pubkey],
            )
            .expect("Close ix to be valid"),
        );
        balance += rent;
    }

    if balance <= 5000 {
        tracing::warn!(
            "Wallet doesn't have enough SOL to withdraw: {} | 5001 at least",
            balance
        );
        return Ok(Some(ixs));
    }

    let amount_to_withdraw = balance - 5000;

    ixs.push(solana_sdk::system_instruction::transfer(
        wallet_pubkey,
        cex_pubkey,
        amount_to_withdraw,
    ));

    Ok(Some(ixs))
}

async fn process_account(provider: &RpcClient, account: &mut Account) -> eyre::Result<()> {
    let wallet = account.keypair();
    let wallet_pubkey = account.get_pubkey();
    let cex_pubkey = Pubkey::from_str(account.get_cex_address())?;

    tracing::info!("Wallet address: `{}`", wallet.pubkey());

    let instructions = match get_ixs(provider, &wallet_pubkey, &cex_pubkey).await? {
        Some(ixs) => ixs,
        None => return Ok(()),
    };

    let (recent_blockhash, _) = provider
        .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
        .await?;

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet_pubkey),
        &[&wallet],
        recent_blockhash,
    );

    send_and_confirm_tx(provider, tx).await?;

    Ok(())
}
