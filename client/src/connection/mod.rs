mod ssh;
pub mod video;

use std::any::type_name;

use anyhow::{Context, Error};
use futures::{SinkExt, StreamExt};
use review_server::{
    config::{StreamConfig, device::DeviceConfig},
    version::VersionInfo,
};
use serde::{Serialize, de::DeserializeOwned};
use tokio::net::TcpStream;
use tokio_util::{
    bytes::{Bytes, BytesMut},
    codec::{Framed, LengthDelimitedCodec},
};
use tracing::info;

use crate::config::ClientOptions;

#[derive(Debug)]
pub struct Connection {
    framed: Framed<TcpStream, LengthDelimitedCodec>,
}

impl Connection {
    pub async fn new(client_options: ClientOptions) -> Result<Self, Error> {
        info!("setting up TCP connection");
        let stream = TcpStream::connect(format!(
            "{}:{}",
            client_options.remarkable_ip, client_options.tcp_port,
        ))
        .await
        .context("could not connect to TCP stream")?;

        stream.set_nodelay(true).context("could not set nodelay")?;

        let framed = Framed::new(stream, LengthDelimitedCodec::new());

        Ok(Connection { framed })
    }

    pub async fn initialize_communication(
        &mut self,
        client_options: ClientOptions,
    ) -> Result<DeviceConfig, Error> {
        self.authenticate(client_options.clone())
            .await
            .context("error while authenticating")?;

        let version_info: VersionInfo = self
            .receive()
            .await
            .context("could not receive version info")?;

        info!("received version information: {}", version_info);

        let device_config = DeviceConfig::new(version_info).context(format!(
            "could not get device configuration for version {}",
            version_info,
        ))?;

        let stream_config = StreamConfig {
            device_config: device_config.clone(),
            framerate: client_options.framerate,
            show_cursor: client_options.show_cursor,
        };

        info!("sending out stream config {:?}", &stream_config);

        self.send(&stream_config)
            .await
            .context("could not send device config")?;

        Ok(device_config)
    }

    async fn receive<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
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

    async fn receive_raw(&mut self) -> Result<BytesMut, Error> {
        self.framed
            .next()
            .await
            .context("connection closed before raw message was received".to_string())?
            .context("could not receive raw message".to_string())
    }

    async fn send<T: Serialize>(&mut self, value: &T) -> Result<(), Error> {
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

    async fn send_raw(&mut self, msg: Bytes) -> Result<(), Error> {
        self.framed
            .send(msg)
            .await
            .context("could not send raw message".to_string())
            .map(|_| ())
    }
}
