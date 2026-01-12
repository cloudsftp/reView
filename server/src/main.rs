mod config;
mod version;

use anyhow::{Context, Error, anyhow};
use clap::Parser;
use config::{CliOptions, ServerOptions, get_video_config};
use tracing::{debug, info};
use version::{get_firmware_version, get_hardware_version};

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let opts = CliOptions::parse();
    debug!("cli options: {:?}", opts);
    let opts = ServerOptions::try_from(opts).context("could not get server options")?;
    debug!("resolved options: {:?}", opts);

    let hardware_version = get_hardware_version().context("could not get hardware version")?;
    info!("Detected hardware version {:?}", hardware_version);

    let firmware_version = get_firmware_version().context("could not get version")?;
    info!("Detected firmware version {:?}", firmware_version);

    let video_config =
        get_video_config(&hardware_version, &firmware_version).context("could not get config")?;
    info!("using video config: {:?}", video_config);

    return Err(anyhow!("test"));

    Ok(())
}
