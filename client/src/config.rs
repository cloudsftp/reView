use std::{
    env::{self, VarError},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, Error};
use clap::Parser;
use tracing::trace;

const DEFAULT_IP: &str = "10.11.99.1";
const DEFAULT_TCP_PORT: u16 = 6680;

const DEFAULT_FRAMERATE: f32 = 50.;

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct CliOptions {
    /// IP of the reMarkable tablet (default: 10.11.99.1)
    #[arg(long, name = "remarkable-ip")]
    remarkable_ip: Option<String>,

    /// TCP port for video stream (default: 6680)
    #[arg(long, name = "tcp-port")]
    tcp_port: Option<u16>,

    /// Private SSH key file path. If not specified, attempting all keys in the SSH directory
    #[arg(long, name = "ssh-key")]
    ssh_key: Option<PathBuf>,

    /// Framerate (default: 50)
    #[arg(long, name = "framerate")]
    framerate: Option<f32>,
    // Dark mode - invert colors (default: false)
    //#[arg(long, name = "dark-mode")]
    //dark_mode: bool,

    // Show cursor (default: false)
    //#[arg(long, name = "show-cursor")]
    //show_cursor: bool,
}

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub remarkable_ip: String,
    pub ssh_key: Option<PathBuf>,
    // TODO: implement dark mode
    // pub dark_mode: bool,
    pub tcp_port: u16,
    //pub show_cursor: bool,
    pub framerate: f32,
}

impl From<CliOptions> for ClientOptions {
    fn from(value: CliOptions) -> Self {
        Self {
            remarkable_ip: resolve_option(
                value.remarkable_ip,
                "REMARKABLE_IP",
                DEFAULT_IP.to_string(),
            ),
            tcp_port: resolve_with(
                value.tcp_port,
                "REMARKABLE_TCP_PORT",
                |string| {
                    string
                        .parse()
                        .context("could not parse TCP port from environment")
                },
                DEFAULT_TCP_PORT,
            ),
            ssh_key: resolve_with_optional(value.ssh_key, "REMARKABLE_SSH_KEY_PATH", |string| {
                PathBuf::from_str(&string).context("could not parse path of private SSH key")
            }),
            framerate: resolve_with(
                value.framerate,
                "REMARKABLE_FRAMERATE",
                |string| {
                    string
                        .parse()
                        .context("could not parse framerate from environment")
                },
                DEFAULT_FRAMERATE,
            ),
            //dark_mode: resolve_boolean_option(value.dark_mode, "REMARKABLE_DARK_MODE", false),
            //show_cursor: resolve_boolean_option(value.show_cursor, "REMARKABLE_SHOW_CURSOR", false),
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

/*
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
*/

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

    trace!(
        "read environment variable '{}': {:?}",
        variable_name, env_string,
    );
    parse(
        env_string
            .unwrap_or_else(|_| panic!("could not get environment varialbe '{}'", variable_name)),
    )
    .unwrap_or_else(|_| panic!("could not parse environemt variable '{}'", variable_name))
}

fn resolve_with_optional<T>(
    cli_value: Option<T>,
    variable_name: &str,
    parse: impl FnOnce(String) -> Result<T, Error>,
) -> Option<T> {
    if let Some(cli_value) = cli_value {
        return Some(cli_value);
    }

    let env_string = env::var(variable_name);
    if let Ok(env_string) = env_string {
        if env_string.is_empty() {
            return None;
        }

        trace!(
            "read environment variable '{}': {}",
            variable_name, env_string,
        );
        return Some(parse(env_string).unwrap_or_else(|_| {
            panic!("could not parse environemt variable '{}'", variable_name)
        }));
    }

    None
}
