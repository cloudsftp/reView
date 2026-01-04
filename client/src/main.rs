use anyhow::{Context, Error};
use async_ssh2_tokio::{AuthMethod, Client, ServerCheckMethod};
use itertools::Itertools;
use log::{debug, info};
use tokio::sync::mpsc;

const HEIGHT: usize = 1872;
const WIDTH: usize = 1404;
const PIXEL_FORMAT: &str = "bgra";
const BYTES_PER_PIXEL: usize = 4;
const FILE: &str = ":mem:";
const SKIP_OFFSET: usize = 2629636;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    run_restream().await?;

    Ok(())
}

async fn run_restream() -> Result<(), Error> {
    info!("connecting to reMarkable");
    let client = Client::connect(
        ("192.168.2.118", 22),
        "root",
        AuthMethod::PrivateKeyFile {
            key_file_path: "/home/fabi/.ssh/id_ed25519".into(),
            key_pass: None,
        },
        ServerCheckMethod::NoCheck,
    )
    .await
    .context("could not connect to reMarkable tablet")?;

    let (stdout_tx, mut stdout_rx) = mpsc::channel(10);
    debug!("spawning restream");
    let exec_future = client.execute_io("./restream --help", stdout_tx, None, None, false, None);

    let mut result_stdout = vec![];

    tokio::pin!(exec_future);
    let result = loop {
        tokio::select! {
            result = &mut exec_future => break result,
            Some(stdout) = stdout_rx.recv() => {
                debug!("ssh stdout: {}", String::from_utf8_lossy(&stdout));
                result_stdout.push(stdout);
            },
        };
    }?;

    debug!("command exited with error code {}", result);
    debug!("sdtout: {:?}", result_stdout.iter().map(|line| String::from_utf8_lossy(&line)).collect_vec());

    /*
    let mut restream = session
        .command("./restream")
        .args(&[
            "--height",
            &HEIGHT.to_string(),
            "--width",
            &WIDTH.to_string(),
            "--bytes-per-pixel",
            &BYTES_PER_PIXEL.to_string(),
            "--file",
            FILE,
            "--skip",
            &SKIP_OFFSET.to_string(),
        ])
        .spawn()
        .await?;
    */

    client.disconnect().await.context("could not disconnect from reMarkable")?;

    Ok(())
}
