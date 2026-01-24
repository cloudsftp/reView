pub mod ssh;
pub mod video;

use std::any::type_name;

use anyhow::{Context, Error};
use futures::{StreamExt, sink::SinkExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

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

    pub async fn exchange_information(&mut self) -> Result<StreamConfig, Error> {
        self.authenticate()
            .await
            .context("error while authenticating client")?;

        let version_info =
            VersionInfo::get_from_device().context("could not get version information")?;

        info!(
            "got version information: hardware {:?}, firmware {}",
            version_info.hardware, version_info.firmware,
        );

        info!("sending out version information");

        self.send(&version_info)
            .await
            .context("could not send out version information")?;

        let stream_config: StreamConfig = self
            .receive()
            .await
            .context("could not receive stream config")?;

        info!("received stream config {:?}", &stream_config);

        Ok(stream_config)
    }

    async fn receive<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let msg = self
            .framed
            .next()
            .await
            .context("connection closed before stream configuration was sent")?
            .context("could not message with stream configuration")?
            .clone();

        let stream_config = bson::deserialize_from_slice(&msg).context(format!(
            "could not deserialize message of type {}",
            type_name::<T>(),
        ))?;

        Ok(stream_config)
    }

    pub async fn send<T: Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let msg = bson::serialize_to_vec(value)
            .context(format!("could not serialize type {}", type_name::<T>()))?;

        self.framed
            .send(msg.into())
            .await
            .context("could not send serialized version information")
            .map(|_| ())
    }
}
