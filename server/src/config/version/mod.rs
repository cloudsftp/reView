#[cfg(test)]
mod tests;

use std::fs;

use anyhow::{Context, Error, Result, anyhow};
use tracing::trace;

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigVersion {
    Ancient,
    V3,
    V3P7,
    V3P24,
}

#[derive(Debug, PartialEq, Eq, Ord)]
pub struct FirmwareVersion {
    version: usize,
    major: usize,
    minor: usize,
    patch: usize,
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

pub fn get_firmware_version() -> Result<(FirmwareVersion, ConfigVersion), Error> {
    let content = fs::read_to_string("/usr/share/remarkable/update.conf")
        .context("could not read framebuffer version file")?;
    let content = content.trim();

    let firmware_version = parse_version(&content).context("could not parse version")?;
    let config_version =
        get_config_version(&firmware_version).context("could not get config version")?;

    Ok((firmware_version, config_version))
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

fn get_config_version(version: &FirmwareVersion) -> Result<ConfigVersion, Error> {
    Ok(if version >= &VERSION_3_24 {
        ConfigVersion::V3P24
    } else if version >= &VERSION_3_7 {
        ConfigVersion::V3P7
    } else if version >= &VERSION_3_0 {
        ConfigVersion::V3
    } else {
        ConfigVersion::Ancient
    })
}

fn parse_version(input: &str) -> Result<FirmwareVersion, Error> {
    let (_, version_string) = input
        .split_once('=')
        .context(format!("could not split input '{}'", input))?;

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

#[derive(Debug)]
pub enum HardwareVersion {
    Rm1,
    Rm2,
    Ferrari,
}

pub fn get_hardware_version() -> Result<HardwareVersion, Error> {
    let content = fs::read_to_string("/sys/devices/soc0/machine")
        .context("could not read framebuffer version file")?;
    let content = content.trim();

    parse_hardware_version(&content).context("could not parse hardware version")
}

fn parse_hardware_version(input: &str) -> Result<HardwareVersion, Error> {
    let (_, version_string) = input
        .split_once(' ')
        .context(format!("could not split input '{}'", input))?;

    match version_string {
        "1.0" => Ok(HardwareVersion::Rm1),
        "2.0" => Ok(HardwareVersion::Rm2),
        "Ferrari" => Ok(HardwareVersion::Ferrari),
        version_string => Err(anyhow!("unknown version string '{}'", version_string)),
    }
}
