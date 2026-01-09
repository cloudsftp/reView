use std::{io::Read as _, net::TcpStream, thread::sleep, time::Duration};

use anyhow::{Context, Error};
use async_ssh2_tokio::{AuthMethod, Client, ServerCheckMethod};
use gstreamer_app::AppSrc;
use lz4_flex::frame::FrameDecoder;
use tokio::sync::mpsc::{self, Receiver};
use tracing::{debug, error, info};

use gstreamer::{Pipeline, prelude::*};
use gstreamer_video::VideoFormat;

const IP: &str = "192.168.0.105";
const SSH_PORT: u16 = 22;
const PORT: u16 = 6680;

const HEIGHT: u32 = 1872;
const WIDTH: u32 = 1404;
const PIXEL_FORMAT: &str = "bgra";
const VIDEO_FORMAT: VideoFormat = VideoFormat::Bgra;
const BYTES_PER_PIXEL: u32 = 4;
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
    let encoded_video_data = TcpStream::connect(format!("{}:{}", IP, PORT))
        .context("could not connect to TCP stream")?;

    let mut decoded_video_data = FrameDecoder::new(encoded_video_data);

    let (pipeline, appsrc) = build_pipeline().context("could not build gstreamer pipeline")?;
    pipeline
        .set_state(gstreamer::State::Playing)
        .context("could not start playing gstreamer pipeline")?;

    let mut chunk = vec![0u8; (BYTES_PER_PIXEL * HEIGHT * WIDTH) as usize];
    loop {
        decoded_video_data
            .read_exact(&mut chunk)
            .context("could not read from TCP stream")?;

        let buffer = gstreamer::Buffer::from_mut_slice(chunk.clone());
        appsrc
            .push_buffer(buffer)
            .context("could not push buffer to app source")?;
    }
}

fn build_pipeline() -> Result<(Pipeline, AppSrc), Error> {
    let video_info = gstreamer_video::VideoInfo::builder(VIDEO_FORMAT, WIDTH, HEIGHT)
        .build()
        .context("could not build video info")?;

    let appsrc = gstreamer_app::AppSrc::builder()
        .caps(
            &video_info
                .to_caps()
                .context("could not get caps from video info")?,
        )
        .is_live(true)
        .format(gstreamer::Format::Time)
        .build();

    let videoconvert = gstreamer::ElementFactory::make("videoconvert").build()?;
    let sink = gstreamer::ElementFactory::make("autovideosink").build()?;

    let pipeline = gstreamer::Pipeline::default();
    pipeline
        .add_many([appsrc.upcast_ref(), &videoconvert, &sink])
        .context("could not add elements to pipeline")?;
    gstreamer::Element::link_many([appsrc.upcast_ref(), &videoconvert, &sink])
        .context("could not link elements toghether")?;

    Ok((pipeline, appsrc))
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
