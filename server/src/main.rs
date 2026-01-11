pub mod config;

use anyhow::{Context, Error};
use config::version::get_version;
use tracing::info;

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    info!("Hello from the server");

    let (firmware_version, config_version) = get_version().context("could not get version")?;

    info!(
        "detected firmware version {:?}, using config version {:?}",
        firmware_version, config_version,
    );

    Ok(())
}
