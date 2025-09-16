use crate::{config::Config, ip2loc::Location};
use anyhow::{Context as _, Result};
use axum::{Json, Router, extract::ConnectInfo, http::HeaderMap, routing::get};
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
    location: Vec<Location>,
}

async fn ip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<IpResponse>, String> {
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|header| header.to_str().ok())
        .map(|header| header.to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    let location = Location::get(&ip).await;

    Ok(Json(IpResponse { ip, location }))
}
