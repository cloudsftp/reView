pub mod video;

use anyhow::{Context, Error};
use futures::{StreamExt, sink::SinkExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::config::StreamConfig;
use crate::version::VersionInfo;

#[derive(Debug)]
pub struct Connection {
    framed: Framed<TcpStream, LengthDelimitedCodec>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let framed = Framed::new(stream, LengthDelimitedCodec::new());

        Connection { framed }
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
