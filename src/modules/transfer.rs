use std::{str::FromStr, time::Duration};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey, signer::Signer,
    transaction::Transaction,
};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    onchain::{
        constants::{TESTME_PUBKEY, TOKEN_PROGRAM_ID},
        derive::derive_ata,
        ixs::create_ata,
        tx::send_and_confirm_tx,
        typedefs::CreateAtaArgs,
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

    let (wallet_token_ata, _) = derive_ata(wallet_pubkey, &TESTME_PUBKEY, &TOKEN_PROGRAM_ID);
    let token_ata_exist = provider.get_account_data(&wallet_token_ata).await.is_ok();

    if token_ata_exist {
        let token_account = provider
            .get_token_account_balance(&wallet_token_ata)
            .await?;

        let token_account_balance = token_account.amount.parse::<u64>()?;

        if token_account_balance != 0 {
            let (cex_token_ata, _) = derive_ata(cex_pubkey, &TESTME_PUBKEY, &TOKEN_PROGRAM_ID);
            let collector_token_ata_exist = provider.get_account_data(&cex_token_ata).await.is_ok();

            if !collector_token_ata_exist {
                let create_ata_args = CreateAtaArgs {
                    funding_address: *wallet_pubkey,
                    associated_account_address: cex_token_ata,
                    wallet_address: *wallet_pubkey,
                    token_mint_address: TESTME_PUBKEY,
                    token_program_id: TOKEN_PROGRAM_ID,
                    instruction: 0,
                };

                ixs.push(create_ata(create_ata_args));
            }

            ixs.push(spl_token::instruction::transfer_checked(
                &TOKEN_PROGRAM_ID,
                &wallet_token_ata,
                &TESTME_PUBKEY,
                &cex_token_ata,
                wallet_pubkey,
                &[wallet_pubkey],
                token_account_balance,
                9u8,
            )?);
        }

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
    }

    let mut balance = provider.get_balance(wallet_pubkey).await?;

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
        balance - 5000,
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
