use modules::menu;
use utils::logger::init_default_logger;

mod config;
mod db;
mod jup_api;
mod me_api;
mod modules;
mod onchain;
mod utils;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = init_default_logger();

    if let Err(e) = menu().await {
        tracing::error!("Execution stopped with an unexpected error: {e}");
    }

    Ok(())
}
