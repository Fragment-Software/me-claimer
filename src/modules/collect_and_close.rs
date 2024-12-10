use std::{str::FromStr, time::Duration};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signer::Signer;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    native_token::{lamports_to_sol, sol_to_lamports},
    program_pack::Pack,
    pubkey::Pubkey,
    transaction::Transaction,
};

use crate::onchain::constants::ME_PUBKEY;
use crate::onchain::crypto::get_wallet;
use crate::{
    config::Config,
    db::{account::Account, database::Database},
    onchain::{
        constants::TOKEN_PROGRAM_ID, derive::derive_ata, ixs::Instructions,
        tx::send_and_confirm_tx, typedefs::CreateAtaArgs,
    },
    utils::misc::pretty_sleep,
};

pub async fn collect_and_close(mut db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    while let Some(account) =
        db.get_random_account_with_filter(|a| !a.get_collected_sol() || !a.get_closed_ata())
    {
        if let Err(e) = process_account(&provider, account, config).await {
            tracing::error!("{}", e);
        } else {
            account.set_closed_ata(true);
            account.set_collected_sol(true);
            db.update();
        };

        pretty_sleep(config.claim_sleep_range).await;
    }

    Ok(())
}

async fn get_ixs(
    provider: &RpcClient,
    wallet_pubkey: &Pubkey,
    collector_pubkey: Option<&str>,
    payer_pubkey: &Pubkey,
    config: &Config,
) -> eyre::Result<Option<Vec<Instruction>>> {
    let mut ixs = vec![];

    let (wallet_token_ata, _) = derive_ata(wallet_pubkey, &ME_PUBKEY, &TOKEN_PROGRAM_ID);
    let token_ata_exist = provider.get_account_data(&wallet_token_ata).await.is_ok();

    let mut should_add_rent = false;

    let rent = provider
        .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
        .await?;

    if token_ata_exist {
        if payer_pubkey == wallet_pubkey {
            should_add_rent = true;
        }

        let token_account = provider
            .get_token_account_balance(&wallet_token_ata)
            .await?;

        let token_account_balance = token_account.amount.parse::<u64>()?;

        if token_account_balance != 0 && config.collect_me {
            let user = Pubkey::from_str(collector_pubkey.unwrap())?;

            let (collector_token_ata, _) = derive_ata(&user, &ME_PUBKEY, &TOKEN_PROGRAM_ID);
            let collector_token_ata_exist = provider
                .get_account_data(&collector_token_ata)
                .await
                .is_ok();

            if !collector_token_ata_exist {
                let create_ata_args = CreateAtaArgs {
                    funding_address: *payer_pubkey,
                    associated_account_address: collector_token_ata,
                    wallet_address: user,
                    token_mint_address: ME_PUBKEY,
                    token_program_id: TOKEN_PROGRAM_ID,
                    instruction: 0,
                };

                ixs.push(Instructions::create_ata(create_ata_args));
            }

            ixs.push(spl_token::instruction::transfer_checked(
                &TOKEN_PROGRAM_ID,
                &wallet_token_ata,
                &ME_PUBKEY,
                &collector_token_ata,
                wallet_pubkey,
                &[wallet_pubkey],
                token_account_balance,
                6u8,
            )?);
        }

        if token_account_balance == 0 {
            let close_ix =
                Instructions::close_account(&wallet_token_ata, wallet_pubkey, payer_pubkey, rent);

            ixs.extend_from_slice(&close_ix);
        }
    }

    if config.collect_sol {
        let to_pubkey = Pubkey::from_str(collector_pubkey.unwrap())?;

        let mut balance = provider.get_balance(wallet_pubkey).await?;

        balance = if should_add_rent {
            balance + rent - sol_to_lamports(lamports_to_sol(rent) * 0.03)
        } else {
            balance
        };

        if balance <= 5000 {
            tracing::warn!(
                "Wallet doesn't have enough SOL to withdraw: {} | 5001 at least",
                balance
            );
            return Ok(Some(ixs));
        }

        let amount_to_withdraw = if payer_pubkey == wallet_pubkey {
            balance - 5000
        } else {
            balance
        };

        ixs.push(solana_sdk::system_instruction::transfer(
            wallet_pubkey,
            &to_pubkey,
            amount_to_withdraw,
        ));
    }

    Ok(Some(ixs))
}

async fn process_account(
    provider: &RpcClient,
    account: &mut Account,
    config: &Config,
) -> eyre::Result<()> {
    let wallet = account.keypair()?;
    let wallet_pubkey = account.get_pubkey();
    let collector_pubkey = account.get_cex_address();

    tracing::info!("Wallet address: `{}`", wallet.pubkey());

    let payer_kp = match config.use_external_fee_pay {
        true => get_wallet(&config.external_fee_payer_secret)?,
        false => wallet.insecure_clone(),
    };

    let signing_keypairs = match config.use_external_fee_pay {
        true => vec![&payer_kp, &wallet],
        false => vec![&wallet],
    };

    let instructions = match get_ixs(
        provider,
        &wallet_pubkey,
        collector_pubkey,
        &payer_kp.pubkey(),
        config,
    )
    .await?
    {
        Some(ixs) => ixs,
        None => return Ok(()),
    };

    if !instructions.is_empty() {
        let (recent_blockhash, _) = provider
            .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
            .await?;

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer_kp.pubkey()),
            &signing_keypairs,
            recent_blockhash,
        );

        send_and_confirm_tx(provider, tx).await?;
    }

    Ok(())
}
