use super::config::*;

use anyhow::{Context, Error};
use async_ssh2_tokio::{AuthMethod, Client, ServerCheckMethod};
use review_server::config::ServerOptions;
use tokio::sync::mpsc::{self, Receiver};
use tracing::{debug, info};

pub async fn connect_ssh(opts: ClientOptions) -> Result<Client, Error> {
    info!("connecting to reMarkable");
    Client::connect(
        (opts.remarkable_ip.clone(), opts.ssh_port),
        "root",
        AuthMethod::PrivateKeyFile {
            key_file_path: opts.ssh_key.clone(),
            key_pass: None,
        },
        ServerCheckMethod::NoCheck,
    )
    .await
    .context("could not connect to reMarkable tablet")
}

pub async fn start_server(
    client: &Client,
    opts: ClientOptions,
) -> Result<
    (
        impl Future<Output = Result<u32, async_ssh2_tokio::Error>>,
        Receiver<Vec<u8>>,
    ),
    Error,
> {
    let (stdout_tx, stdout_rx) = mpsc::channel(10);

    let server_options = ServerOptions {
        port: 6680,
        show_cursor: false,
        framerate: 10,
    };

    let restream_command = Box::leak(Box::new(format!(
        "RUST_LOG=trace ./review-server '{}'",
        serde_json::to_string(&server_options)
            .context("could not convert server options to json")?,
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
