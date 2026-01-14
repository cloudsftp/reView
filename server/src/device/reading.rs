use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use anyhow::{Context, Error, anyhow};
use itertools::Itertools;
use procfs::process::{MMapPath, all_processes};

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
            config::VideoDataSource::Memory => {
                let processes = all_processes()
                    .context("could not get process iterator")?
                    .filter_map(|p| {
                        let p = p.ok()?;
                        (p.stat().ok()?.comm == "xochitl").then_some(p)
                    })
                    .collect_vec();

                if processes.len() != 1 {
                    return Err(anyhow!(
                        "expected exactly 1 xochitl process, found {}",
                        processes.len(),
                    ));
                }
                let process = processes.first().expect("just checked vector length");

                let memory_file = process.mem().context("could not get xochitl memory file")?;

                let framebuffer_path_name =
                    MMapPath::from("/dev/fb0").context("could not build framebuffer path name")?;

                let maps = process.maps().context("could not get process maps")?;
                let maps = maps
                    .iter()
                    .filter(|m| m.pathname == framebuffer_path_name)
                    .collect_vec();

                if maps.len() != 1 {
                    return Err(anyhow!(
                        "expected exactly 1 xochitl memory maps, found {}",
                        maps.len(),
                    ));
                }
                let framebuffer_map = maps.first().expect("just checked vector length");
                let offset = framebuffer_map.address.0 as usize;

                (memory_file, offset)
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
