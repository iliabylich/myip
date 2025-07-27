use anyhow::{Context as _, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) port: u16,
}

#[cfg(debug_assertions)]
const PATH: &str = "config.toml";

#[cfg(not(debug_assertions))]
const PATH: &str = "/etc/myip/config.toml";

impl Config {
    pub(crate) async fn read() -> Result<Self> {
        let contents = tokio::fs::read_to_string(PATH)
            .await
            .with_context(|| format!("failed to read config from {PATH}"))?;

        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("failed to parse copnfig from {PATH}"))?;

        Ok(config)
    }
}
