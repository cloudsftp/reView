mod config;
mod display;
mod start;

use anyhow::{Context, Error};
use start::{connect_ssh, receive_output, start_server};
use tracing::error;

use display::gstreamer_thread;
use gstreamer::{Pipeline, prelude::*};
use gstreamer_video::VideoFormat;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = connect_ssh()
        .await
        .context("could not connect to reMarkable")?;
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
