use super::config::*;

use std::path::PathBuf;

use anyhow::{Context, Error};
use async_ssh2_tokio::{AuthMethod, Client, ServerCheckMethod};
use tokio::sync::mpsc::{self, Receiver};
use tracing::{debug, info};

pub async fn connect_ssh(
    key_file_path: PathBuf,
    remarkable_ip: Option<String>,
) -> Result<Client, Error> {
    info!("connecting to reMarkable");
    Client::connect(
        (remarkable_ip.unwrap_or(DEFAULT_IP.into()), SSH_PORT),
        "root",
        AuthMethod::PrivateKeyFile {
            key_file_path,
            key_pass: None,
        },
        ServerCheckMethod::NoCheck,
    )
    .await
    .context("could not connect to reMarkable tablet")
}

pub async fn start_server(
    client: &Client,
) -> Result<
    (
        impl Future<Output = Result<u32, async_ssh2_tokio::Error>>,
        Receiver<Vec<u8>>,
    ),
    Error,
> {
    let (stdout_tx, stdout_rx) = mpsc::channel(10);

    let restream_command = Box::leak(Box::new(format!(
        "./restream --height {} --width {} --bytes-per-pixel {} --file {} --skip {} --listen {}",
        HEIGHT, WIDTH, BYTES_PER_PIXEL, FILE, SKIP_OFFSET, PORT,
    )));

    debug!("spawning restream");
    let exec_future = client.execute_io(restream_command, stdout_tx, None, None, false, None);

    Ok((exec_future, stdout_rx))
}

pub async fn receive_output(stdout: &mut Receiver<Vec<u8>>) -> Result<String, Error> {
    let mut buf = vec![];
    while let Some(mut data) = stdout.recv().await {
        buf.append(&mut data);
    }
    return Ok(String::from_utf8_lossy(&buf).to_string());
}
