use anyhow::{Context, Error};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CliOptions {
    /// JSON object containing the server configuration
    payload: String,
}

/// Configurable options for the reView server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerOptions {
    /// Port to listen for the TCP connection
    pub port: u16,
    /// Wheter the cursor should be visualized
    pub show_cursor: bool,
    /// The framerate in frames per second
    pub framerate: f32,
}

impl TryFrom<CliOptions> for ServerOptions {
    type Error = Error;

    fn try_from(value: CliOptions) -> Result<Self, Error> {
        serde_json::from_str(&value.payload)
            .context("could not parse JSON payload for server options")
    }
}
