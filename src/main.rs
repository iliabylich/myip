use anyhow::{Context as _, Result};
use axum::{Json, Router, extract::ConnectInfo, routing::get};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let Args { port } = Args::parse();

    let app = Router::new().route("/", get(ip));

    let listener = TcpListener::bind(("127.0.0.1", port))
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

struct Args {
    port: u16,
}

fn print_help_and_exit() -> ! {
    const HELP: &str = "Usage: myip --port <port>";
    eprintln!("{}", HELP);
    std::process::exit(1)
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);

        let mut port = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--port" => {
                    port = Some(
                        args.next()
                            .unwrap_or_else(|| print_help_and_exit())
                            .parse::<u16>()
                            .unwrap_or_else(|err| {
                                eprintln!("{err:?}");
                                print_help_and_exit();
                            }),
                    )
                }

                other => {
                    eprintln!("unknown option {other:?}");
                    print_help_and_exit()
                }
            }
        }

        Self {
            port: port.unwrap_or_else(|| print_help_and_exit()),
        }
    }
}
