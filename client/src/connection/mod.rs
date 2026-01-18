use anyhow::{Context, Error};
use futures::{SinkExt, StreamExt};
use review_server::{
    config::{StreamConfig, device::DeviceConfig},
    version::VersionInfo,
};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
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
            client_options.remarkable_ip, client_options.tcp_port
        ))
        .await
        .context("could not connect to TCP stream")?;

        let framed = Framed::new(stream, LengthDelimitedCodec::new());

        Ok(Connection { framed })
    }

    pub async fn receive_version_info(&mut self) -> Result<VersionInfo, Error> {
        let msg = self
            .framed
            .next()
            .await
            .context("connection was dropped before version info was communicated")?
            .context("could not receive version info message")?;

        let version_info =
            bson::deserialize_from_slice(&msg).context("could not deserialize version info")?;

        Ok(version_info)
    }

    pub async fn send_stream_config(&mut self, stream_config: StreamConfig) -> Result<(), Error> {
        let msg =
            bson::serialize_to_vec(&stream_config).context("could not serialize stream config")?;

        self.framed
            .send(msg.into())
            .await
            .context("could not send stream config")
            .map(|_| ())
    }
}

/*

fn to_video_format(pixel_format: &PixelFormat) -> VideoFormat {
    match pixel_format {
        PixelFormat::Rgb565le => todo!("not sure what the video format for RGB 565 LE is"),
        PixelFormat::Gray8 => VideoFormat::Gray8,
        PixelFormat::Gray16be => VideoFormat::Gray16Be,
        PixelFormat::Bgra => VideoFormat::Bgra,
    }
}

*/
