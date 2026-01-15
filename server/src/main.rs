mod config;
mod device;
mod version;

use std::fs::OpenOptions;

use anyhow::{Context, Error};
use clap::Parser;
use config::{CliOptions, ServerOptions, VideoConfig, get_video_config};
use tracing::{debug, info};
use tracing_subscriber::{Registry, fmt, layer::SubscriberExt};
use version::{get_firmware_version, get_hardware_version};

use crate::{
    device::connection::listen_for_clients,
    version::{FirmwareVersion, HardwareVersion},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    initialize_logging().context("could not initialize logging")?;

    let opts = get_command_line_options().context("could not read command line options")?;
    let (hardware_version, firmware_version, video_config) =
        get_versions_and_config().context("could not gather version infos and video config")?;

    listen_for_clients(hardware_version, firmware_version, video_config, opts)
        .await
        .context("problem while listening for connections")?;

    Ok(())
}

fn initialize_logging() -> Result<(), Error> {
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
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
    let opts = CliOptions::parse();
    debug!("cli options: {:?}", opts);
    let opts = ServerOptions::try_from(opts).context("could not get server options")?;
    debug!("resolved options: {:?}", opts);

    Ok(opts)
}

fn get_versions_and_config() -> Result<(HardwareVersion, FirmwareVersion, VideoConfig), Error> {
    let hardware_version = get_hardware_version().context("could not get hardware version")?;
    info!("Detected hardware version {:?}", hardware_version);

    let firmware_version = get_firmware_version().context("could not get version")?;
    info!("Detected firmware version {:?}", firmware_version);

    let video_config =
        get_video_config(&hardware_version, &firmware_version).context("could not get config")?;
    info!("using video config: {:?}", video_config);

    Ok((hardware_version, firmware_version, video_config))
}
