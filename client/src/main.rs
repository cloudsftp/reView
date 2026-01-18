mod config;
mod connection;
mod display;

use anyhow::{Context, Error};
use clap::Parser;
use config::{CliOptions, ClientOptions};
use connection::{Connection, video::VideoConnection};
use review_server::config::{StreamConfig, device::DeviceConfig};
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

    let version_info = conn
        .receive_version_info()
        .await
        .context("could not receive version info")?;

    info!("received version information: {}", version_info);

    let device_config = DeviceConfig::new(version_info).context(format!(
        "could not get device configuration for version {}",
        version_info,
    ))?;

    let stream_config = StreamConfig {
        device_config: device_config.clone(),
        framerate: client_options.framerate,
        show_cursor: client_options.show_cursor,
    };

    info!("sending out stream config {:?}", &stream_config);

    conn.send_stream_config(stream_config)
        .await
        .context("could not send device config")?;

    let mut video_connection = VideoConnection::new(conn, device_config.video_config)
        .context("could not initialize video connection")?;
    video_connection
        .run()
        .await
        .context("error while streaming")?;

    Ok(())
}
