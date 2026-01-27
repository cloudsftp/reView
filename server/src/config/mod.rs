pub mod device;

use clap::Parser;
use device::DeviceConfig;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CliOptions {
    /// Port to listen for the TCP connections (default: 6680)
    #[arg(long, name = "port", default_value = "6680")]
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamConfig {
    pub device_config: DeviceConfig,
    pub framerate: f32,
    pub show_cursor: bool,
}
