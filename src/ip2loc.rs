use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{task::JoinSet, time::timeout};

#[derive(Serialize, Debug)]
pub(crate) struct Location {
    lat: f64,
    lng: f64,
    source: Source,
}

#[derive(Serialize, Debug)]
pub(crate) enum Source {
    #[serde(rename = "freegeoip")]
    FreeGeoIP,
    #[serde(rename = "ipapi")]
    IpAPI,
    #[serde(rename = "ipwhois")]
    IpWhoIs,
}

impl Location {
    pub(crate) async fn get(ip: &str) -> Vec<Self> {
        let mut set = JoinSet::new();

        macro_rules! add {
            ($t:tt) => {
                set.spawn({
                    let ip = ip.to_string();
                    async move { $t::get(ip).await }
                })
            };
        }

        add!(FreeGeoIP);
        add!(IpAPI);
        add!(IpWhoIs);

        set.join_all()
            .await
            .into_iter()
            .filter_map(|e| e.ok())
            .collect()
    }
}

trait Service {
    const SOURCE: Source;

    async fn get_lat_lng(ip: String) -> Result<(f64, f64)>;

    async fn get(ip: String) -> Result<Location> {
        timeout(Duration::from_secs(1), Self::get_lat_lng(ip))
            .await
            .context("timeout error")?
            .map(|(lat, lng)| Location {
                lat,
                lng,
                source: Self::SOURCE,
            })
    }
}

struct FreeGeoIP;
impl Service for FreeGeoIP {
    const SOURCE: Source = Source::FreeGeoIP;

    async fn get_lat_lng(ip: String) -> Result<(f64, f64)> {
        let url = format!("https://freegeoip.app/json/{}", ip);
        #[derive(Deserialize, Debug)]
        struct Response {
            latitude: f64,
            longitude: f64,
        }

        let response = reqwest::get(url).await?.json::<Response>().await?;
        Ok((response.latitude, response.longitude))
    }
}

struct IpAPI;
impl Service for IpAPI {
    const SOURCE: Source = Source::IpAPI;

    async fn get_lat_lng(ip: String) -> Result<(f64, f64)> {
        let url = format!("http://ip-api.com/json/{}", ip);
        #[derive(Deserialize, Debug)]
        struct Response {
            lat: f64,
            lon: f64,
        }
        let response = reqwest::get(url).await?.json::<Response>().await?;
        Ok((response.lat, response.lon))
    }
}

struct IpWhoIs;
impl Service for IpWhoIs {
    const SOURCE: Source = Source::IpWhoIs;

    async fn get_lat_lng(ip: String) -> Result<(f64, f64)> {
        let url = format!("http://ipwhois.app/json/{}", ip);

        #[derive(Deserialize, Debug)]
        struct Response {
            latitude: f64,
            longitude: f64,
        }
        let response = reqwest::get(url).await?.json::<Response>().await?;
        Ok((response.latitude, response.longitude))
    }
}
