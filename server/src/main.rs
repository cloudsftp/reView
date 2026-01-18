mod config;
mod connection;
mod device;
mod version;

use std::fs::OpenOptions;

use anyhow::{Context, Error};
use clap::Parser;
use config::CliOptions;
use tracing::info;
use tracing_subscriber::{Registry, fmt, layer::SubscriberExt};

use connection::Connection;
use version::VersionInfo;

#[tokio::main]
async fn main() -> Result<(), Error> {
    initialize_logging().context("could not initialize logging")?;

    let cli_options = CliOptions::parse();

    let version_info =
        VersionInfo::get_from_device().context("could not get version information")?;

    info!(
        "got version information: hardware {:?}, firmware {}",
        version_info.hardware, version_info.firmware,
    );
    info!("initializing TCP connection");

    let mut conn = Connection::new(cli_options)
        .await
        .context("error while handling TCP connection")?;

    info!("sending out version information");

    conn.send_version_info(version_info)
        .await
        .context("could not send out version information")?;

    let stream_config = conn
        .receive_stream_config()
        .await
        .context("could not receive stream config")?;

    info!("received stream config {:?}", &stream_config);

    // TODO: set up video streaming

    Ok(())
}

// TODO: since now running as service, needed?
fn initialize_logging() -> Result<(), Error> {
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./review.log")
        .context("could not open log file")?;

    let subscriber = Registry::default()
        .with(fmt::layer().with_writer(log_file))
        .with(fmt::layer().with_ansi(true).compact());

    tracing::subscriber::set_global_default(subscriber)
        .context("could not set global subscriber")?;

    Ok(())
}
