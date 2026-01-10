use std::{
    env::{self, VarError},
    path::PathBuf,
};

use anyhow::{Context, Error};
use clap::Parser;
use gstreamer_video::VideoFormat;

// TODO: get this info from reMarkable tablet directly
// first parse firmware version (on server)
// second set all settings accordingly
// third send out height, width, pixel format, and bytes per pixel to client
pub const HEIGHT: u32 = 1872;
pub const WIDTH: u32 = 1404;
pub const PIXEL_FORMAT: &str = "bgra";
pub const VIDEO_FORMAT: VideoFormat = VideoFormat::Bgra;
pub const BYTES_PER_PIXEL: u32 = 4;
pub const FILE: &str = ":mem:";
pub const SKIP_OFFSET: usize = 2629636;

const DEFAULT_IP: &str = "10.11.99.1";
const DEFAULT_SSH_PORT: u16 = 22;
const DEFAULT_TCP_PORT: u16 = 6680;

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CliOptions {
    /// IP of the reMarkable tablet (default: 10.11.99.1)
    #[arg(long, name = "remarkable-ip")]
    remarkable_ip: Option<String>,

    /// SSH Port used by the reMarkable tablet (default: 22)
    #[arg(long, name = "ssh-port")]
    ssh_port: Option<u16>,

    /// Private SSH key file path
    #[arg(long, name = "ssh-key")]
    ssh_key: Option<PathBuf>,

    /// TCP port for video stream (default: 6680)
    #[arg(long, name = "tcp-port")]
    tcp_port: Option<u16>,

    /// Dark mode - invert colors (default: false)
    #[arg(long, name = "dark-mode")]
    dark_mode: bool,
}

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub remarkable_ip: String,
    pub ssh_port: u16,
    pub ssh_key: PathBuf,
    pub tcp_port: u16,
    pub dark_mode: bool,
}

impl From<CliOptions> for ClientOptions {
    fn from(value: CliOptions) -> Self {
        Self {
            remarkable_ip: resolve_option(
                value.remarkable_ip,
                "REMARKABLE_IP",
                DEFAULT_IP.to_string(),
            ),
            ssh_port: resolve_with(
                value.ssh_port,
                "REMARKABLE_SSH_PORT",
                |string| string.parse().context("could not parse"),
                DEFAULT_SSH_PORT,
            ),
            ssh_key: must_resolve_option(value.ssh_key, "REMARKABLE_SSH_KEY_PATH"),
            tcp_port: resolve_with(
                value.tcp_port,
                "REMARKABLE_TCP_PORT",
                |string| string.parse().context("could not parse"),
                DEFAULT_TCP_PORT,
            ),
            dark_mode: resolve_boolean_option(value.dark_mode, "REMARKABLE_DARK_MODE", false),
        }
    }
}

fn resolve_option<T: From<String>>(cli_value: Option<T>, variable_name: &str, default: T) -> T {
    resolve_with(
        cli_value,
        variable_name,
        |env_value| Ok(env_value.into()),
        default,
    )
}

fn must_resolve_option<T: From<String>>(cli_value: Option<T>, variable_name: &str) -> T {
    must_resolve_with(cli_value, variable_name, |env_value| Ok(env_value.into()))
}

fn resolve_boolean_option(cli_value: bool, variable_name: &str, default: bool) -> bool {
    let cli_value = if cli_value { Some(true) } else { None };
    resolve_with(
        cli_value,
        variable_name,
        |string| {
            Ok(match string.as_str() {
                "1" => true,
                "true" => true,
                "TRUE" => true,
                "false" => false,
                "FALSE" => false,
                "" => false,
                string => panic!("unknown boolean value: {}", string),
            })
        },
        default,
    )
}

fn resolve_with<T>(
    cli_value: Option<T>,
    variable_name: &str,
    parse: impl FnOnce(String) -> Result<T, Error>,
    default: T,
) -> T {
    if let Some(cli_value) = cli_value {
        return cli_value;
    }

    let env_string = env::var(variable_name);
    if let Err(VarError::NotPresent) = env_string {
        return default;
    }

    parse(env_string.expect(&format!(
        "could not get environment varialbe '{}'",
        variable_name,
    )))
    .expect(&format!(
        "could not parse environemt variable '{}'",
        variable_name,
    ))
}

fn must_resolve_with<T>(
    cli_value: Option<T>,
    variable_name: &str,
    parse: impl FnOnce(String) -> Result<T, Error>,
) -> T {
    if let Some(cli_value) = cli_value {
        return cli_value;
    }

    let env_string = env::var(variable_name).expect(&format!(
        "could not get environment varialbe '{}'",
        variable_name,
    ));
    parse(env_string).expect(&format!(
        "could not parse environemt variable '{}'",
        variable_name,
    ))
}
