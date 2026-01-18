use std::time::Duration;

use anyhow::{Context, Error};
use futures::SinkExt;
use lz4_flex::compress_prepend_size;
use tokio::time::{MissedTickBehavior, interval};
use tracing::{debug, trace};

use super::Connection;
use crate::{config::StreamConfig, device::reading::FrameReader};

#[derive(Debug)]
pub struct VideoConnection {
    conn: Connection,
    stream_config: StreamConfig,
    frame_reader: FrameReader,
}

impl VideoConnection {
    pub fn new(conn: Connection, stream_config: StreamConfig) -> Result<Self, Error> {
        let frame_reader = FrameReader::new(stream_config.device_config.clone())
            .context("could not create frame reader")?;

        Ok(Self {
            conn,
            stream_config,
            frame_reader,
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let mut interval = interval(Duration::from_secs_f32(1. / self.stream_config.framerate));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut buffer = vec![0u8; self.frame_reader.frame_length()];
        loop {
            interval.tick().await;

            self.frame_reader
                .read_frame(&mut buffer)
                .context("error reading frame from file")?;

            debug!("read {} bytes from frame reader", buffer.len());

            let encoded_buffer = compress_prepend_size(&buffer);
            trace!(
                "writing encoded bytes to stream (length {})",
                encoded_buffer.len(),
            );

            self.conn
                .framed
                .send(encoded_buffer.into())
                .await
                .context("could not write frame to encoder")?;

            debug!("wrote the data to the output stream");
        }
    }
}
