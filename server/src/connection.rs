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

        let (stream, addr) = listener.accept().await?;
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

/*
async fn open_connection(
    stream: TcpStream,
    opts: ServerOptions,
    video_config: VideoConfig,
    communicated_config: CommunicatedConfig,
) -> Result<(), Error> {
    let mut frame_reader =
        FrameReader::new(video_config).context("could not create frame reader")?;

    debug!("created frame reader, starting loop to send data");

    let mut interval = interval(Duration::from_secs_f64(1. / (opts.framerate as f64)));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut buffer = vec![0u8; frame_reader.frame_length()];
    loop {
        interval.tick().await;

        frame_reader
            .read_frame(&mut buffer)
            .context("error reading frame from file")?;

        debug!("read {} bytes from frame reader", buffer.len());

        let encoded_buffer = compress_prepend_size(&buffer);
        trace!(
            "writing encoded bytes to stream (length {})",
            encoded_buffer.len(),
        );

        framed
            .send(encoded_buffer.into())
            .await
            .context("could not write frame to encoder")?;

        debug!("wrote the data to the output stream");
    }
}
*/
