use std::path::PathBuf;

use crate::version::{ConfigVersion, FirmwareVersion, HardwareVersion, get_config_version};
use anyhow::{Context, Error, anyhow};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CliOptions {
    /// JSON object containing the server configuration
    payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerOptions {
    pub port: u16,
    pub show_cursor: bool,
}

impl TryFrom<CliOptions> for ServerOptions {
    type Error = Error;

    fn try_from(value: CliOptions) -> Result<Self, Error> {
        serde_json::from_str(&value.payload)
            .context("could not parse JSON payload for server options")
    }
}

#[derive(Debug)]
pub enum VideoDataSource {
    File { path: PathBuf },
    Memory,
}

#[derive(Debug)]
pub enum PixelFormat {
    Rgb565le,
    Gray8,
    Gray16be,
    Bgra,
}

#[derive(Debug)]
pub struct InternalVideoConfig {
    pub source: VideoDataSource,
    pub skip: usize,
}

#[derive(Debug)]
pub struct SharedVideoConfig {
    pub height: usize,
    pub width: usize,
    pub bytes_per_pixel: usize,
    pub pixel_format: PixelFormat,
}

#[derive(Debug)]
pub struct VideoConfig {
    pub internal: InternalVideoConfig,
    pub shared: SharedVideoConfig,
}

pub fn get_video_config(
    hardware_version: &HardwareVersion,
    firmware_version: &FirmwareVersion,
) -> Result<VideoConfig, Error> {
    match hardware_version {
        HardwareVersion::Rm1 => Ok(VideoConfig {
            internal: InternalVideoConfig {
                source: VideoDataSource::File {
                    path: PathBuf::from("/dev/fb0"),
                },
                skip: 8,
            },
            shared: SharedVideoConfig {
                height: 1408,
                width: 1872,
                bytes_per_pixel: 2,
                pixel_format: PixelFormat::Rgb565le,
            },
        }),
        HardwareVersion::Rm2 => {
            let height = 1872;
            let width = 1404;

            match get_config_version(&firmware_version).context("could not get config version")? {
                ConfigVersion::Ancient => Err(anyhow!(
                    "no known configuration values for reMarkable 2 with firmware version {}",
                    firmware_version,
                )),
                ConfigVersion::V3 => Ok(VideoConfig {
                    internal: InternalVideoConfig {
                        source: VideoDataSource::Memory,
                        skip: 8,
                    },
                    shared: SharedVideoConfig {
                        height,
                        width,
                        bytes_per_pixel: 4,
                        pixel_format: PixelFormat::Gray8,
                    },
                }),
                ConfigVersion::V3P7 => Ok(VideoConfig {
                    internal: InternalVideoConfig {
                        source: VideoDataSource::Memory,
                        skip: 8,
                    },
                    shared: SharedVideoConfig {
                        height,
                        width,
                        bytes_per_pixel: 2,
                        pixel_format: PixelFormat::Gray16be,
                    },
                }),
                ConfigVersion::V3P24 => Ok(VideoConfig {
                    internal: InternalVideoConfig {
                        source: VideoDataSource::Memory,
                        skip: 2629636,
                    },
                    shared: SharedVideoConfig {
                        height,
                        width,
                        bytes_per_pixel: 4,
                        pixel_format: PixelFormat::Bgra,
                    },
                }),
            }
        }
        HardwareVersion::Ferrari => Err(anyhow!(
            "no known configuration values known for reMarkable Paper Pro"
        )),
    }
}
