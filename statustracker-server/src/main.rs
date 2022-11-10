pub mod hour;
pub mod name_to_uuid;
pub mod server;
pub mod tracker;
pub mod utils;

use std::path::PathBuf;

use color_eyre::eyre::{eyre, Result};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::tracker::StatusTracker;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = dotenvy::dotenv();
    tracing_subscriber::registry()
        .with(fmt::layer().compact())
        .with(EnvFilter::from_env("RUST_LOG"))
        .init();

    let path = if let Some(path) = std::env::args().nth(1) {
        path.parse::<PathBuf>()?
    } else {
        PathBuf::from("./statustracker.toml")
    };
    let file = std::fs::read(&path).map_err(|e| eyre!("Error opening {}: {e}", path.display()))?;

    server::start_server(StatusTracker::new(toml::from_slice(&file)?).await?).await?;
    Ok(())
}
