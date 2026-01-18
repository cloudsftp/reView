pub mod device;

use clap::Parser;
use device::DeviceConfig;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CliOptions {
    /// Port to listen for the TCP connections
    #[arg(long, name = "port")]
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamConfig {
    pub device_config: DeviceConfig,
    pub framerate: f32,
    pub show_cursor: bool,
}
