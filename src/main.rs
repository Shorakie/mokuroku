mod client;
mod commands;
mod config;
mod events;

use eyre::{eyre, Context, Result};

use crate::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    color_eyre::install()?;

    // reads RUST_LOG env
    tracing_subscriber::fmt::init();

    let mut mokuroku = Client::new()
        .await
        .wrap_err(eyre!("Could not initialize Discord client."))?;

    mokuroku
        .start()
        .await
        .wrap_err(eyre!("Fatality! Mokuroku crashed 💀"))?;

    Ok(())
}
