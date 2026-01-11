pub mod config;

use anyhow::{Context, Error};
use config::version::{get_firmware_version, get_hardware_version};
use tracing::info;

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    info!("Hello from the server");

    let hardware_version = get_hardware_version().context("could not get hardware version")?;
    info!("Detected hardware version {:?}", hardware_version,);

    let (firmware_version, config_version) =
        get_firmware_version().context("could not get version")?;
    info!(
        "Detected firmware version {:?}, using config version {:?}",
        firmware_version, config_version,
    );

    Ok(())
}
