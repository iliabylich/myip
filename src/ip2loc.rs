use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{sync::mpsc::unbounded_channel, time::timeout};

#[derive(Serialize, Debug)]
pub(crate) struct Location {
    pub(crate) lat: f64,
    pub(crate) lng: f64,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub(crate) enum LocationService {
    #[serde(rename = "freegeoip")]
    FreeGeoIP,
    #[serde(rename = "ipapi")]
    IpAPI,
    #[serde(rename = "ipwhois")]
    IpWhoIs,
}

impl LocationService {
    async fn get(self, ip: &str) -> Result<Location> {
        match self {
            LocationService::FreeGeoIP => freegeoip(ip).await,
            LocationService::IpAPI => ipapi(ip).await,
            LocationService::IpWhoIs => ipwhois(ip).await,
        }
    }
}

impl Location {
    pub(crate) async fn get(ip: &str, services: Vec<LocationService>) -> Result<Self> {
        let (tx, mut rx) = unbounded_channel::<Location>();

        let handles = services
            .into_iter()
            .map(|service| {
                let ip = ip.to_string();
                let tx = tx.clone();

                tokio::spawn(async move {
                    if let Ok(loc) = service.get(&ip).await {
                        let _ = tx.send(loc);
                    }
                })
            })
            .collect::<Vec<_>>();

        let location = timeout(Duration::from_secs(1), rx.recv())
            .await
            .context("timeout error")?
            .context("faile to retrieve location")?;

        for handle in handles {
            handle.abort();
        }

        Ok(location)
    }
}

async fn freegeoip(ip: &str) -> Result<Location> {
    let url = format!("https://freegeoip.app/json/{}", ip);
    #[derive(Deserialize, Debug)]
    struct Response {
        latitude: f64,
        longitude: f64,
    }
    let response = reqwest::get(url)
        .await
        .context("failed to send freegeoip request")?
        .json::<Response>()
        .await
        .context("failed to read freegeoip response body")?;

    Ok(Location {
        lat: response.latitude,
        lng: response.longitude,
    })
}

async fn ipapi(ip: &str) -> Result<Location> {
    let url = format!("http://ip-api.com/json/{}", ip);
    #[derive(Deserialize, Debug)]
    struct Response {
        lat: f64,
        lon: f64,
    }
    let response = reqwest::get(url)
        .await
        .context("failed to send ip-api request")?
        .json::<Response>()
        .await
        .context("failed to read ip-api response body")?;

    Ok(Location {
        lat: response.lat,
        lng: response.lon,
    })
}

async fn ipwhois(ip: &str) -> Result<Location> {
    let url = format!("http://ipwhois.app/json/{}", ip);

    #[derive(Deserialize, Debug)]
    struct Response {
        latitude: f64,
        longitude: f64,
    }
    let response = reqwest::get(url)
        .await
        .context("failed to send ipwhois request")?
        .json::<Response>()
        .await
        .context("failed to read ipwhois response body")?;

    Ok(Location {
        lat: response.latitude,
        lng: response.longitude,
    })
}
