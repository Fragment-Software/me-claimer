mod claimer;
mod collect_and_close;
mod prepare_txs;
mod sender;

use crate::{config::Config, db::database::Database};

use claimer::claim_me;
use collect_and_close::collect_and_close;
use dialoguer::{theme::ColorfulTheme, Select};
use sender::sender;

const LOGO: &str = r#"
    ___                                                  __
  /'___\                                                /\ \__
 /\ \__/  _ __    __       __     ___ ___      __    ___\ \ ,_\
 \ \ ,__\/\`'__\/'__`\   /'_ `\ /' __` __`\  /'__`\/' _ `\ \ \/
  \ \ \_/\ \ \//\ \L\.\_/\ \L\ \/\ \/\ \/\ \/\  __//\ \/\ \ \ \_
   \ \_\  \ \_\\ \__/.\_\ \____ \ \_\ \_\ \_\ \____\ \_\ \_\ \__\
    \/_/   \/_/ \/__/\/_/\/___L\ \/_/\/_/\/_/\/____/\/_/\/_/\/__/
                  ___  __  /\____/
                /'___\/\ \_\_/__/
   ____    ___ /\ \__/\ \ ,_\ __  __  __     __    _ __    __
  /',__\  / __`\ \ ,__\\ \ \//\ \/\ \/\ \  /'__`\ /\`'__\/'__`\
 /\__, `\/\ \L\ \ \ \_/ \ \ \\ \ \_/ \_/ \/\ \L\.\\ \ \//\  __/
 \/\____/\ \____/\ \_\   \ \__\ \___x___/'\ \__/.\_\ \_\\ \____\
  \/___/  \/___/  \/_/    \/__/\/__//__/   \/__/\/_/\/_/ \/____/

                     t.me/fragment_software
"#;

pub async fn menu() -> eyre::Result<()> {
    let config = Config::read_default().await;

    println!("{LOGO}");

    loop {
        let options = vec![
            "Generate a database for a session",
            "Claim $ME",
            "Send SOL from payer to claim wallets",
            "Collect $ME + Close $ME ATA + Collect SOL",
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
                let _ = Database::new(&config).await?;
                tracing::info!("Database successfully generated")
            }
            1 => {
                let db = Database::read().await;
                claim_me(db, &config).await?;
            }
            2 => {
                let db = Database::read().await;
                sender(db, &config).await?;
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
