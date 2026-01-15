use std::io::{self, Read, Write};
use std::time::Duration;

use anyhow::{Context, Error};
use futures::sink::SinkExt;
use lz4_flex::frame::FrameEncoder;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info};

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

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("new connection from {}", addr);

        tokio::spawn(open_connection(
            stream,
            opts.clone(),
            video_config.clone(),
            communicated_config.clone(),
        ));
    }
}

async fn open_connection(
    stream: TcpStream,
    opts: ServerOptions,
    video_config: VideoConfig,
    communicated_config: CommunicatedConfig,
) -> Result<(), Error> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

    let bytes = bson::serialize_to_vec(&communicated_config)
        .context("could not serialize communicated config")?
        .into();

    framed
        .send(bytes)
        .await
        .context("could not send out config")?;

    let stream = framed
        .into_inner()
        .into_std()
        .context("could not turn stream into std")?;

    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .context("could not set write timeout")?;

    let mut encoded_video_data = FrameEncoder::new(stream);
    let mut frame_reader =
        FrameReader::new(video_config).context("could not create frame reader")?;

    debug!("created frame reader, starting loop to send data");

    io::copy(&mut frame_reader, &mut encoded_video_data)
        .context("error while copying frame buffer data to stream")?;

    Ok(())

    // TODO: this problably can be a simple io::copy
    /*
    let mut buffer = vec![0u8; frame_reader.frame_length()];
    loop {
        frame_reader
            .read_one_frame(&mut buffer)
            .context("error reading frame from file")?;

        debug!("read exactly {} bytes from frame reader", buffer.len());

        encoded_video_data
            .write_all(&buffer)
            .context("could not write frame to encoder")?;

        debug!("wrote the data to the encoder");

        encoded_video_data
            .flush()
            .context("failed to flush encoder")?;

        debug!("flushed the encoded video data");

        sleep(Duration::from_secs_f64(1. / (opts.framerate as f64))).await;
    }
    */
}
