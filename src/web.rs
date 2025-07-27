use crate::config::Config;
use anyhow::{Context as _, Result};
use axum::{Json, Router, extract::ConnectInfo, routing::get};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub(crate) async fn start(config: Config) -> Result<()> {
    let app = Router::new().route("/", get(ip));

    let listener = TcpListener::bind(("127.0.0.1", config.port))
        .await
        .context("failed to bind")?;
    println!(
        "Listening on {}",
        listener.local_addr().context("failed to get local addr")?
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("Failed to spawn web server")?;

    Ok(())
}

#[derive(Serialize)]
struct IpResponse {
    ip: String,
}

async fn ip(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> Json<IpResponse> {
    Json(IpResponse {
        ip: addr.ip().to_string(),
    })
}
