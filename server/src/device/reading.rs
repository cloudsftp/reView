use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use anyhow::{Context, Error, anyhow};
use tracing::trace;

use super::process::get_xochitl_memory_file;
use crate::config::{self, VideoConfig};

#[derive(Debug)]
pub struct FrameReader {
    file: File,
    offset: usize,
    width: usize,
    height: usize,
    bytes_per_pixel: usize,
}

impl FrameReader {
    pub fn new(video_config: VideoConfig) -> Result<Self, Error> {
        let (file, offset) = match video_config.internal.source {
            config::VideoDataSource::File { path } => (
                File::open(path).context("could not open framebuffer file")?,
                0,
            ),
            config::VideoDataSource::ProcessMemory => get_xochitl_memory_file()
                .context("could not get file and offset for xochitl process")?,
        };

        trace!(
            "file offset: {}, extra skip: {}",
            offset, video_config.internal.skip
        );
        let offset = offset + video_config.internal.skip;

        Ok(Self {
            file,
            offset,
            width: video_config.shared.width,
            height: video_config.shared.height,
            bytes_per_pixel: video_config.shared.bytes_per_pixel,
        })
    }

    #[inline]
    pub fn frame_length(&self) -> usize {
        self.width * self.height * self.bytes_per_pixel
    }

    pub fn read_frame(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        if buf.len() != self.frame_length() {
            return Err(anyhow!(
                "called read_frame with buffer of length {}, expected length {}",
                buf.len(),
                self.frame_length(),
            ));
        }

        self.file
            .seek(SeekFrom::Start(self.offset as u64))
            .context("could not point file to beginning of frame")?;
        self.file
            .read_exact(buf)
            .context("could not read frame from file")?;

        Ok(())
    }
}
