use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use anyhow::{Context, Error};

use super::process::get_memory_file;
use crate::config::{self, VideoConfig};

#[derive(Debug)]
pub struct FrameReader {
    file: File,
    offset: usize,
    current: usize,
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
            config::VideoDataSource::ProcessMemory => {
                get_memory_file().context("could not get file and offset for xochitl process")?
            }
        };

        let mut frame_reader = Self {
            file,
            offset,
            current: 0,
            width: video_config.shared.width,
            height: video_config.shared.height,
            bytes_per_pixel: video_config.shared.bytes_per_pixel,
        };
        frame_reader
            .point_file_to_framebuffer_memory_start()
            .context("could not initialize file to offset")?;
        Ok(frame_reader)
    }

    pub fn frame_length(&self) -> usize {
        self.width * self.height * self.bytes_per_pixel
    }

    // TODO: anyhow error handling instead of io Errors
    fn point_file_to_framebuffer_memory_start(&mut self) -> std::io::Result<()> {
        self.file.seek(SeekFrom::Start(self.offset as u64))?;
        self.current = 0;

        Ok(())
    }
}

impl Read for FrameReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let requested = buf.len();
        let bytes_read = if self.current + requested < self.frame_length() {
            self.file.read(buf)?
        } else {
            let rest = self.frame_length() - self.current;
            self.file.read(&mut buf[0..rest])?
        };

        self.current += bytes_read;
        if self.current == self.frame_length() {
            self.point_file_to_framebuffer_memory_start()?;
        }
        Ok(bytes_read)
    }
}
