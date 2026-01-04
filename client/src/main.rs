use std::{process::Stdio, thread};

use anyhow::{Context, Error};
use async_ssh2_tokio::{AuthMethod, Client, ServerCheckMethod};
use itertools::Itertools as _;
use log::{debug, info};
use tokio::{io::AsyncWriteExt, process::Command, sync::mpsc};

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

    let mut command = Command::new("ffmpeg");
    command.args(&[
        "-vcodec",
        "rawvideo",
        "-f",
        "rawvideo",
        "-pixel_format",
        PIXEL_FORMAT,
        "-video_size",
        &format!("{},{}", WIDTH, HEIGHT),
        "-i",
        "-",
        "/tmp/test.mkv",
    ]);
    command.stdin(Stdio::piped());
    let mut command = command.spawn().context("could not spawn ffmpeg command")?;
    let mut stdin = command
        .stdin
        .take()
        .context("could not get stdin of ffmpeg")?;

    thread::spawn(async move || {
        debug!("spawning restream");
        let command = format!(
            "./restream --height {} --width {} --bytes-per-pixel {} --file {} --skip {}",
            HEIGHT, WIDTH, BYTES_PER_PIXEL, FILE, SKIP_OFFSET,
        );
        let exec_future = client.execute_io(&command, stdout_tx, None, None, false, None);
        tokio::pin!(exec_future);
        loop {
            tokio::select! {
                result = &mut exec_future => break result,
                Some(stdout) = stdout_rx.recv() => {
                    //debug!("ssh stdout: {}", String::from_utf8_lossy(&stdout));
                    debug!("read some bytes (length: {})", stdout.len());
                    stdin.write(&stdout).await.unwrap();
                },
            };
        }
    });



    /*
    debug!("command exited with error code {}", result);
    debug!("sdtout: {:?}", result_stdout.iter().map(|line| String::from_utf8_lossy(&line)).collect_vec());
    */

    //client.disconnect().await.context("could not disconnect from reMarkable")?;

    Ok(())
}
