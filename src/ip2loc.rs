use std::time::Duration;

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    task::JoinHandle,
    time::timeout,
};

#[derive(Serialize, Debug)]
pub(crate) struct Location {
    pub(crate) lat: f64,
    pub(crate) lng: f64,
}

impl Location {
    pub(crate) async fn get(ip: &str) -> Result<Self> {
        let (tx, mut rx) = unbounded_channel::<Location>();

        let h1 = spawn::<FreeGeoIP>(ip, tx.clone());
        let h2 = spawn::<IpAPI>(ip, tx.clone());
        let h3 = spawn::<IpWhoIs>(ip, tx.clone());

        let location = timeout(Duration::from_secs(1), rx.recv())
            .await
            .context("timeout error")?
            .context("faile to retrieve location")?;

        h1.abort();
        h2.abort();
        h3.abort();

        Ok(location)
    }
}

fn spawn<S: Service>(ip: &str, tx: UnboundedSender<Location>) -> JoinHandle<()> {
    let ip = ip.to_string();

    tokio::spawn(async move {
        if let Ok(loc) = S::get(&ip).await {
            let _ = tx.send(loc);
        }
    })
}

#[async_trait::async_trait]
trait Service {
    async fn get(ip: &str) -> Result<Location>;
}

struct FreeGeoIP;
#[async_trait::async_trait]
impl Service for FreeGeoIP {
    async fn get(ip: &str) -> Result<Location> {
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
}

struct IpAPI;
#[async_trait::async_trait]
impl Service for IpAPI {
    async fn get(ip: &str) -> Result<Location> {
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
}

struct IpWhoIs;
#[async_trait::async_trait]
impl Service for IpWhoIs {
    async fn get(ip: &str) -> Result<Location> {
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
}
