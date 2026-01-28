pub mod ssh;
pub mod video;

use std::any::type_name;

use anyhow::{Context, Error};
use futures::{StreamExt, sink::SinkExt};
use serde::{Serialize, de::DeserializeOwned};
use tokio::net::TcpStream;
use tokio_util::bytes::{Bytes, BytesMut};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

use crate::config::StreamConfig;
use crate::version::VersionInfo;

#[derive(Debug)]
pub struct Connection {
    framed: FramedTcpConnection,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let framed = FramedTcpConnection::new(stream);

        Self { framed }
    }

    pub async fn initialize_communication(&mut self) -> Result<StreamConfig, Error> {
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

        self.framed
            .send(&version_info)
            .await
            .context("could not send out version information")?;

        let stream_config: StreamConfig = self
            .framed
            .receive()
            .await
            .context("could not receive stream config")?;

        info!("received stream config {:?}", &stream_config);

        Ok(stream_config)
    }
}

#[derive(Debug)]
pub struct FramedTcpConnection {
    framed: Framed<TcpStream, LengthDelimitedCodec>,
}

impl FramedTcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        let framed = Framed::new(stream, LengthDelimitedCodec::new());

        Self { framed }
    }

    pub async fn receive<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let msg = self
            .framed
            .next()
            .await
            .context(format!(
                "connection closed before message of type {} was received",
                type_name::<T>(),
            ))?
            .context(format!(
                "could not receive message of type {}",
                type_name::<T>()
            ))?;

        let stream_config = bson::deserialize_from_slice(&msg).context(format!(
            "could not deserialize message of type {}",
            type_name::<T>(),
        ))?;

        Ok(stream_config)
    }

    pub async fn receive_raw(&mut self) -> Result<BytesMut, Error> {
        self.framed
            .next()
            .await
            .context("connection closed before raw message was received".to_string())?
            .context("could not receive raw message".to_string())
    }

    pub async fn send<T: Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let msg = bson::serialize_to_vec(value)
            .context(format!("could not serialize type {}", type_name::<T>()))?;

        self.framed
            .send(msg.into())
            .await
            .context(format!(
                "could not send serialized message of type {}",
                type_name::<T>()
            ))
            .map(|_| ())
    }

    pub async fn send_raw(&mut self, msg: Bytes) -> Result<(), Error> {
        self.framed
            .send(msg)
            .await
            .context("could not send raw message".to_string())
            .map(|_| ())
    }
}
