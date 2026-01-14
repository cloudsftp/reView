mod config;
mod version;

use std::{
    fs::File,
    io::{self, ErrorKind, Read, Seek, SeekFrom, Write},
    time::Duration,
};

use anyhow::{Context, Error, anyhow};
use clap::Parser;
use config::{
    CliOptions, CommunicatedConfig, ServerOptions, VersionInfo, VideoConfig, get_video_config,
};
use futures::sink::SinkExt;
use itertools::Itertools;
use lz4_flex::frame::FrameEncoder;
use procfs::process::{MMapPath, all_processes};
use tokio::{
    net::{TcpListener, TcpStream},
    time::sleep,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info};
use version::{get_firmware_version, get_hardware_version};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let opts = CliOptions::parse();
    debug!("cli options: {:?}", opts);
    let opts = ServerOptions::try_from(opts).context("could not get server options")?;
    debug!("resolved options: {:?}", opts);

    let hardware_version = get_hardware_version().context("could not get hardware version")?;
    info!("Detected hardware version {:?}", hardware_version);

    let firmware_version = get_firmware_version().context("could not get version")?;
    info!("Detected firmware version {:?}", firmware_version);

    let video_config =
        get_video_config(&hardware_version, &firmware_version).context("could not get config")?;
    info!("using video config: {:?}", video_config);

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

    io::copy(&mut frame_reader, &mut encoded_video_data)
        .context("error while copying frame buffer data to stream")?;

    // TODO: this problably can be a simple io::copy
    let mut buffer = vec![0u8; frame_reader.frame_length()];
    loop {
        frame_reader
            .read_exact(&mut buffer)
            .context("error reading frame from file")?;

        encoded_video_data
            .write_all(&buffer)
            .context("could not write frame to encoder")?;

        encoded_video_data
            .flush()
            .context("failed to flush encoder")?;

        sleep(Duration::from_secs_f64(1. / (opts.framerate as f64))).await;
    }
}

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

    fn frame_length(&self) -> usize {
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
