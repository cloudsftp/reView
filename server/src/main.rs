mod config;
mod connection;
mod device;
mod version;

use std::fs::OpenOptions;

use anyhow::{Context, Error};
use clap::Parser;
use config::server::{CliOptions, ServerOptions};
use tracing::{debug, info};
use tracing_subscriber::{Registry, fmt, layer::SubscriberExt};

use connection::Connection;
use version::VersionInfo;

#[tokio::main]
async fn main() -> Result<(), Error> {
    initialize_logging().context("could not initialize logging")?;

    let server_options =
        get_command_line_options().context("could not read command line options")?;

    let version_info =
        VersionInfo::get_from_device().context("could not get version information")?;

    info!(
        "got version information: hardware {:?}, firmware {}",
        version_info.hardware, version_info.firmware,
    );
    info!("initializing TCP connection");

    let mut conn = Connection::new(server_options)
        .await
        .context("error while handling TCP connection")?;

    info!("sending out version information");

    conn.send_version_info(version_info)
        .await
        .context("could not send out version information")?;

    let device_config = conn
        .receive_device_config()
        .await
        .context("could not receive device config")?;

    info!("received device config {:?}", &device_config);

    // TODO: set up video streaming

    Ok(())
}

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

fn get_command_line_options() -> Result<ServerOptions, Error> {
    let cli_options = CliOptions::parse();
    debug!("cli options: {:?}", cli_options);
    let server_options =
        ServerOptions::try_from(cli_options).context("could not get server options")?;
    debug!("resolved options: {:?}", server_options);

    Ok(server_options)
}
