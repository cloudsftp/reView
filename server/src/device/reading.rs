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

    pub fn frame_length(&self) -> usize {
        self.width * self.height * self.bytes_per_pixel
    }

    pub fn read_one_frame(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        trace!("attempting to read one frame");
        if buf.len() != self.frame_length() {
            return Err(anyhow!(
                "frame is {} bytes long, but buffer is only {} bytes long",
                self.frame_length(),
                buf.len(),
            ));
        }

        trace!("pointing file to start of frame: {}", self.offset);
        self.file
            .seek(SeekFrom::Start(self.offset as u64))
            .context("could not point file to beginning of frame")?;
        trace!("reading one frame from memory: {} bytes", buf.len());
        self.file
            .read_exact(buf)
            .context(format!("could not read {} bytes from memory", buf.len()))
            .map(|_| {
                trace!("successfully read one frame from memory");

                ()
            })
    }
}
