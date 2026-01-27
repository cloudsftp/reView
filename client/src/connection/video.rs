use anyhow::{Context, Error};
use lz4_flex::decompress_size_prepended;
use tracing::trace;

use crate::display::Display;

use super::Connection;
use review_server::config::device::VideoConfig;

#[derive(Debug)]
pub struct VideoConnection {
    conn: Connection,
    display: Display,
}

impl VideoConnection {
    pub fn new(conn: Connection, video_config: VideoConfig) -> Result<Self, Error> {
        let display = Display::new(video_config).context("could not initialize display")?;

        Ok(Self { conn, display })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            trace!("attempting to read data from TCP stream");

            let compressed_frame = self
                .conn
                .receive_raw()
                .await
                .context("could not reveive next frame")?;

            trace!(
                "read one compressed frame from TCP stream ({} bytes)",
                compressed_frame.len(),
            );

            let frame = decompress_size_prepended(&compressed_frame)
                .context("could not decompress received frame")?;

            trace!("decompressed: {} bytes", frame.len());

            self.display
                .push_frame(frame)
                .context("could not push frame to display")?;
        }
    }
}
