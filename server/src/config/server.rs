use anyhow::Error;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CliOptions {
    /// Port to listen for the TCP connections
    #[arg(long, name = "port")]
    pub port: u16,
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
        Ok(Self {
            port: value.port,
            // TODO: show cursor and framerate communicated over tcp, not here
            show_cursor: false,
            framerate: 50.,
        })
    }
}
