mod claimer;
mod swap;
mod transfer;

use crate::{config::Config, db::database::Database};

use claimer::claim_testme;
use dialoguer::{theme::ColorfulTheme, Select};
use swap::swap_testme;
use transfer::collect_and_close;

pub async fn menu() -> eyre::Result<()> {
    let config = Config::read_default().await;

    loop {
        let options = vec![
            "Generate a database for a session",
            "Claim $TestME",
            "Swap $TestME -> $USDC",
            "Close $TestMe ata + Transfer $USDC",
            "Exit",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choice:")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => {
                let _ = Database::new().await?;
                tracing::info!("Database successfully generated")
            }
            1 => {
                let db = Database::read().await;
                claim_testme(db, &config).await?;
            }
            2 => {
                let db = Database::read().await;
                swap_testme(db, &config).await?;
            }
            3 => {
                let db = Database::read().await;
                collect_and_close(db, &config).await?;
            }
            4 => {
                return Ok(());
            }
            _ => tracing::error!("Invalid selection"),
        }
    }
}
