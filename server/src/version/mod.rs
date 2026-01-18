#[cfg(test)]
mod tests;

use core::fmt;
use std::{fs, str::FromStr};

use anyhow::{Context, Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::trace;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HardwareVersion {
    Rm1,
    Rm2,
    Ferrari,
}

const HARDWARE_VERSION_FILE: &str = "/sys/devices/soc0/machine";

impl HardwareVersion {
    pub fn get_from_device() -> Result<Self, Error> {
        let content = fs::read_to_string(HARDWARE_VERSION_FILE)
            .context("could not read framebuffer version file")?;
        let content = content.trim();

        content.parse().context("could not parse hardware version")
    }
}

impl FromStr for HardwareVersion {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (_, version_string) = s
            .split_once(' ')
            .context(format!("could not split input '{}'", s))?;

        match version_string {
            "1.0" => Ok(HardwareVersion::Rm1),
            "2.0" => Ok(HardwareVersion::Rm2),
            "Ferrari" => Ok(HardwareVersion::Ferrari),
            version_string => Err(anyhow!("unknown version string '{}'", version_string)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub struct FirmwareVersion {
    pub version: usize,
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl PartialOrd for FirmwareVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.version.partial_cmp(&other.version) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.major.partial_cmp(&other.major) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.minor.partial_cmp(&other.minor) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.patch.partial_cmp(&other.patch)
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}.{}",
            self.version, self.major, self.minor, self.patch
        ))
    }
}

const FIRMWARE_VERSION_FILE: &str = "/usr/share/remarkable/update.conf";

impl FirmwareVersion {
    pub fn get_from_device() -> Result<Self, Error> {
        let content = fs::read_to_string(FIRMWARE_VERSION_FILE)
            .context("could not read framebuffer version file")?;
        let content = content.trim();

        content
            .parse()
            .context(format!("could not parse firmware version {}", content))
    }
}

impl FromStr for FirmwareVersion {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (_, version_string) = s
            .split_once('=')
            .context(format!("could not split input '{}'", s))?;

        trace!("version string: {}", version_string);

        let version_parts = version_string
            .split('.')
            .map(|s| {
                s.parse()
                    .context(format!("could not parse '{}' as usize", s))
            })
            .collect::<Result<Vec<usize>, Error>>()?;

        trace!("got parts: {:?}", version_parts);

        if version_parts.len() > 4 {
            return Err(anyhow!("too many version parts: {:?}", version_parts));
        }

        Ok(FirmwareVersion {
            version: version_parts[0],
            major: version_parts[1],
            minor: version_parts[2],
            patch: version_parts[3],
        })
    }
}

// TODO: include server version?
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VersionInfo {
    pub hardware: HardwareVersion,
    pub firmware: FirmwareVersion,
}

impl VersionInfo {
    pub fn get_from_device() -> Result<Self, Error> {
        Ok(Self {
            hardware: HardwareVersion::get_from_device()?,
            firmware: FirmwareVersion::get_from_device()?,
        })
    }
}
