use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use anyhow::{Context, Error, anyhow};
use tracing::trace;

use super::process::get_xochitl_memory_file;
use crate::config::device::{DeviceConfig, FramebufferDataSource};

#[derive(Debug)]
pub struct FrameReader {
    file: File,
    offset: usize,
    width: usize,
    height: usize,
    bytes_per_pixel: usize,
}

impl FrameReader {
    pub fn new(device_config: DeviceConfig) -> Result<Self, Error> {
        let (file, offset) = match device_config.framebuffer_config.source {
            FramebufferDataSource::File { path } => (
                File::open(path).context("could not open framebuffer file")?,
                0,
            ),
            FramebufferDataSource::ProcessMemory => get_xochitl_memory_file()
                .context("could not get file and offset for xochitl process")?,
        };

        trace!(
            "file offset: {}, extra skip: {}",
            offset, device_config.framebuffer_config.skip,
        );
        let offset = offset + device_config.framebuffer_config.skip;

        Ok(Self {
            file,
            offset,
            width: device_config.video_config.width,
            height: device_config.video_config.height,
            bytes_per_pixel: device_config.video_config.pixel_format.bytes_per_pixel(),
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
