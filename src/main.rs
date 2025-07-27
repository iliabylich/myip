mod config;
mod web;

use anyhow::Result;
use config::Config;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Config::read().await?;
    web::start(config).await?;

    Ok(())
}
