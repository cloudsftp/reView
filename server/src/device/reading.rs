use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use anyhow::{Context, Error};
use tracing::trace;

use super::process::get_xochitl_memory_file;
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
            config::VideoDataSource::ProcessMemory => get_xochitl_memory_file()
                .context("could not get file and offset for xochitl process")?,
        };

        trace!(
            "file offset: {}, extra skip: {}",
            offset, video_config.internal.skip
        );
        let offset = offset + video_config.internal.skip;

        let mut frame_reader = Self {
            file,
            offset,
            current: 0,
            width: video_config.shared.width,
            height: video_config.shared.height,
            bytes_per_pixel: video_config.shared.bytes_per_pixel,
        };

        frame_reader
            .point_file_to_beginning_of_frame()
            .context("could not point file to beginning of frame")?;

        Ok(frame_reader)
    }

    pub fn frame_length(&self) -> usize {
        self.width * self.height * self.bytes_per_pixel
    }

    fn point_file_to_beginning_of_frame(&mut self) -> std::io::Result<()> {
        self.file.seek(SeekFrom::Start(self.offset as u64))?;
        self.current = 0;

        Ok(())
    }
}

impl Read for FrameReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = buf.len();

        trace!("requested to read {} bytes from process", n);

        let n = if self.current + n < self.frame_length() {
            trace!(
                "reading next {} bytes from process (current: {}, end: {})",
                n,
                self.current,
                self.frame_length(),
            );

            self.file.read_exact(buf)?;
            self.current += n;

            n
        } else {
            let n = self.frame_length() - self.current;

            trace!(
                "reading last {} bytes from process (current: {}, end: {})",
                n,
                self.current,
                self.frame_length(),
            );

            self.file.read_exact(&mut buf[..n])?;

            self.point_file_to_beginning_of_frame()?;

            n
        };

        trace!("read exactly {} bytes from process", n);

        Ok(n)
    }
}
