mod config;
mod connection;
mod display;

use anyhow::{Context, Error};
use clap::Parser;
use config::{CliOptions, ClientOptions};
use connection::{Connection, video::VideoConnection};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let cli_options = CliOptions::parse();
    debug!("cli options: {:?}", cli_options);
    let client_options = ClientOptions::from(cli_options);
    debug!("resolved options: {:?}", &client_options);

    info!(
        "connecting to reMarkable tablet at {}:{}",
        client_options.remarkable_ip, client_options.tcp_port,
    );

    let mut conn = Connection::new(client_options.clone())
        .await
        .context("could not initialize TCP connection")?;

    let device_config = conn
        .exchange_information(client_options.clone())
        .await
        .context("error during initial information exchange")?;

    let mut video_connection = VideoConnection::new(conn, device_config.video_config)
        .context("could not initialize video connection")?;
    video_connection
        .run()
        .await
        .context("error while streaming")?;

    Ok(())
}
