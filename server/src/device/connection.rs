use std::time::Duration;

use anyhow::{Context, Error};
use futures::sink::SinkExt;
use lz4_flex::compress_prepend_size;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{MissedTickBehavior, interval};
use tokio_util::bytes::Bytes;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info, trace};

use crate::device::reading::FrameReader;
use crate::{
    config::{CommunicatedConfig, ServerOptions, VersionInfo, VideoConfig},
    version::{FirmwareVersion, HardwareVersion},
};

pub async fn listen_for_clients(
    hardware_version: HardwareVersion,
    firmware_version: FirmwareVersion,
    video_config: VideoConfig,
    opts: ServerOptions,
) -> Result<(), Error> {
    let communicated_config = CommunicatedConfig {
        version: VersionInfo {
            hardware: hardware_version,
            firmware: firmware_version,
        },
        video_config: video_config.shared.clone(),
    };

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", opts.port))
        .await
        .context(format!("could not bind to port {}", opts.port))?;

    let (stream, addr) = listener.accept().await?;
    info!("new connection from {}", addr);

    open_connection(
        stream,
        opts.clone(),
        video_config.clone(),
        communicated_config.clone(),
    )
    .await
    .context("error while handling TCP connections")?;

    Ok(())
}

async fn open_connection(
    stream: TcpStream,
    opts: ServerOptions,
    video_config: VideoConfig,
    communicated_config: CommunicatedConfig,
) -> Result<(), Error> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

    let bytes: Bytes = bson::serialize_to_vec(&communicated_config)
        .context("could not serialize communicated config")?
        .into();

    framed
        .send(bytes.iter().copied().collect())
        .await
        .context("could not send out config")?;

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
