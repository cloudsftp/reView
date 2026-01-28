mod ssh;
pub mod video;

use anyhow::{Context, Error};
use review_server::{
    config::{StreamConfig, device::DeviceConfig},
    connection::FramedTcpConnection,
    version::VersionInfo,
};
use tokio::net::TcpStream;
use tracing::info;

use crate::config::ClientOptions;

#[derive(Debug)]
pub struct Connection {
    framed: FramedTcpConnection,
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

        let framed = FramedTcpConnection::new(stream);

        Ok(Self { framed })
    }

    pub async fn initialize_communication(
        &mut self,
        client_options: ClientOptions,
    ) -> Result<DeviceConfig, Error> {
        self.authenticate(client_options.clone())
            .await
            .context("error while authenticating")?;

        let version_info: VersionInfo = self
            .framed
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
            //show_cursor: client_options.show_cursor,
            show_cursor: false,
        };

        info!("sending out stream config {:?}", &stream_config);

        self.framed
            .send(&stream_config)
            .await
            .context("could not send device config")?;

        Ok(device_config)
    }
}
