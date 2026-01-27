use std::path::PathBuf;

use anyhow::{Error, anyhow};
use serde::{Deserialize, Serialize};

use crate::version::{FirmwareVersion, HardwareVersion, VersionInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FramebufferDataSource {
    File { path: PathBuf },
    ProcessMemory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FramebufferConfig {
    pub source: FramebufferDataSource,
    pub skip: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgb565le,
    Gray8,
    Gray16be,
    Bgra,
}

impl PixelFormat {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Rgb565le => 2,
            PixelFormat::Gray8 => 1,
            PixelFormat::Gray16be => 2,
            PixelFormat::Bgra => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub height: usize,
    pub width: usize,
    // TODO: redundant, use function on PixelFormat struct to determine number of bytes used
    pub bytes_per_pixel: usize,
    pub pixel_format: PixelFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub framebuffer_config: FramebufferConfig,
    pub video_config: VideoConfig,
}

impl DeviceConfig {
    #[allow(unused)]
    pub fn new(version_info: VersionInfo) -> Result<Self, Error> {
        let height = 1872;
        let width = 1404;

        match version_info.hardware {
            HardwareVersion::Rm1 => Ok(DeviceConfig {
                framebuffer_config: FramebufferConfig {
                    source: FramebufferDataSource::File {
                        path: PathBuf::from("/dev/fb0"),
                    },
                    skip: 8,
                },

                video_config: VideoConfig {
                    height,
                    width,
                    bytes_per_pixel: 2,
                    pixel_format: PixelFormat::Rgb565le,
                },
            }),
            HardwareVersion::Rm2 => {
                let rm2_config_version = DeviceConfigVersion::from(version_info.firmware);

                match rm2_config_version {
                    DeviceConfigVersion::Ancient => Err(anyhow!(
                        "no known configuration values for reMarkable 2 with firmware version {}",
                        version_info.firmware,
                    )),
                    DeviceConfigVersion::V3 => Ok(DeviceConfig {
                        framebuffer_config: FramebufferConfig {
                            source: FramebufferDataSource::ProcessMemory,
                            skip: 8,
                        },
                        video_config: VideoConfig {
                            height,
                            width,
                            bytes_per_pixel: 1,
                            pixel_format: PixelFormat::Gray8,
                        },
                    }),
                    DeviceConfigVersion::V3P7 => Ok(DeviceConfig {
                        framebuffer_config: FramebufferConfig {
                            source: FramebufferDataSource::ProcessMemory,
                            skip: 8,
                        },
                        video_config: VideoConfig {
                            height,
                            width,
                            bytes_per_pixel: 2,
                            pixel_format: PixelFormat::Gray16be,
                        },
                    }),
                    DeviceConfigVersion::V3P24 => Ok(DeviceConfig {
                        framebuffer_config: FramebufferConfig {
                            source: FramebufferDataSource::ProcessMemory,
                            skip: 2629636,
                        },
                        video_config: VideoConfig {
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
}

const VERSION_3_0: FirmwareVersion = FirmwareVersion {
    version: 3,
    major: 0,
    minor: 0,
    patch: 0,
};
const VERSION_3_7: FirmwareVersion = FirmwareVersion {
    version: 3,
    major: 7,
    minor: 0,
    patch: 1930,
};
const VERSION_3_24: FirmwareVersion = FirmwareVersion {
    version: 3,
    major: 24,
    minor: 0,
    patch: 0,
};

#[derive(Debug, PartialEq, Eq)]
pub enum DeviceConfigVersion {
    Ancient,
    V3,
    V3P7,
    V3P24,
}

impl From<FirmwareVersion> for DeviceConfigVersion {
    fn from(value: FirmwareVersion) -> Self {
        if value >= VERSION_3_24 {
            DeviceConfigVersion::V3P24
        } else if value >= VERSION_3_7 {
            DeviceConfigVersion::V3P7
        } else if value >= VERSION_3_0 {
            DeviceConfigVersion::V3
        } else {
            DeviceConfigVersion::Ancient
        }
    }
}
