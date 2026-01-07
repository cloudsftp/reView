use std::{thread::sleep, time::Duration};

use anyhow::{Context, Error};
use async_ssh2_tokio::{AuthMethod, Client, ServerCheckMethod};
use tokio::{
    io::AsyncReadExt as _, net::TcpStream, sync::mpsc::{self, Receiver}
};
use tracing::{debug, error, info};

use gstreamer::prelude::*;
use gstreamer_video::prelude::*;

const IP: &str = "192.168.2.118";
const SSH_PORT: u16 = 22;
const PORT: u16 = 6680;

const HEIGHT: usize = 1872;
const WIDTH: usize = 1404;
const PIXEL_FORMAT: &str = "bgra";
const BYTES_PER_PIXEL: usize = 4;
const FILE: &str = ":mem:";
const SKIP_OFFSET: usize = 2629636;

const APP_SOURCE_NAME: &str = "binsource";

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = connect_ssh().await?;
    let (restream_command_future, mut restream_command_stdout) = start_server(&client).await?;

    let mut tcp_task = tokio::spawn(gstreamer_thread());

    tokio::pin!(restream_command_future);
    loop {
        tokio::select! {
            restream_exit_code = &mut restream_command_future => {
                error!("restream command exited with code {}", restream_exit_code.context("could not execute restream command")?);

                let restream_output = receive_output(&mut restream_command_stdout).await?;
                error!("stdout+stderr: (next line)\n\n{}\n", restream_output);
            },
            gstreamer_result = &mut tcp_task => {
                error!("gstreamer exited with result: {:?}", gstreamer_result);
            }
        }
    }
}

async fn gstreamer_thread() -> Result<(), Error> {
    gstreamer::init().context("could not init gstreamer")?;

    sleep(Duration::from_millis(100));

    info!("setting up TCP connection");
    let mut tcp_input_stream = TcpStream::connect(format!("{}:{}", IP, PORT))
        .await
        .context("could not connect to TCP stream")?;


    // TODO: rewrite in bindings?
    let pipeline = gstreamer::parse::launch(&format!(
        "appsrc name={} is-live=true format=time ! rawvideoparse width={} height={} format={} ! videoconvert ! autovideosink",
        APP_SOURCE_NAME, WIDTH, HEIGHT, PIXEL_FORMAT, 
    )).context("could not build gstreamer pipeline")?;

    let pipeline = pipeline.dynamic_cast::<gstreamer::Pipeline>().unwrap();
    let app_source = pipeline.by_name(APP_SOURCE_NAME).unwrap()
        .dynamic_cast::<gstreamer_app::AppSrc>().unwrap(); // TODO: don't depend on dyanamic cast

    pipeline.set_state(gstreamer::State::Playing).unwrap();

    let mut chunk = vec![0u8; BYTES_PER_PIXEL * HEIGHT * WIDTH];

    // TODO: tokio::select! for the love of god
    loop {
        if tcp_input_stream.read_exact(&mut chunk).await.is_ok() {
            let buffer = gstreamer::Buffer::from_mut_slice(chunk.clone());
            let _ = app_source.push_buffer(buffer);
        } else {
            break; 
        }
    }

    Ok(())
}

async fn receive_output(stdout: &mut Receiver<Vec<u8>>) -> Result<String, Error> {
    let mut buf = vec![];
    while let Some(mut data) = stdout.recv().await {
        buf.append(&mut data);
    }
    return Ok(String::from_utf8_lossy(&buf).to_string());
}

async fn connect_ssh() -> Result<Client, Error> {
    info!("connecting to reMarkable");
    Client::connect(
        (IP, SSH_PORT),
        "root",
        AuthMethod::PrivateKeyFile {
            key_file_path: "/home/fabi/.ssh/id_ed25519".into(),
            key_pass: None,
        },
        ServerCheckMethod::NoCheck,
    )
    .await
    .context("could not connect to reMarkable tablet")
}

async fn start_server(
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

/*
    let mut command = Command::new("ffplay");
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
    ]);
*/
