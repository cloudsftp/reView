pub mod video;

use anyhow::{Context, Error};
use futures::{StreamExt, sink::SinkExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

use crate::config::{CliOptions, StreamConfig};
use crate::version::VersionInfo;

#[derive(Debug)]
pub struct Connection {
    framed: Framed<TcpStream, LengthDelimitedCodec>,
}

impl Connection {
    pub async fn new(cli_options: CliOptions) -> Result<Self, Error> {
        let listener = TcpListener::bind(&format!("0.0.0.0:{}", cli_options.port))
            .await
            .context(format!("could not bind to port {}", cli_options.port))?;

        let (stream, addr) = listener
            .accept()
            .await
            .context("error while waiting for a TCP connection")?;
        info!("new connection from {}", addr);

        let framed = Framed::new(stream, LengthDelimitedCodec::new());

        Ok(Connection { framed })
    }

    pub async fn send_version_info(&mut self, version_info: VersionInfo) -> Result<(), Error> {
        let msg = bson::serialize_to_vec(&version_info)
            .context("could not serialize version information")?;

        self.framed
            .send(msg.into())
            .await
            .context("could not send serialized version information")
            .map(|_| ())
    }

    pub async fn receive_stream_config(&mut self) -> Result<StreamConfig, Error> {
        let msg = self
            .framed
            .next()
            .await
            .context("connection closed before stream configuration was sent")?
            .context("could not message with stream configuration")?;

        let stream_config =
            bson::deserialize_from_slice(&msg).context("could not deserialize stream config")?;

        Ok(stream_config)
    }
}
