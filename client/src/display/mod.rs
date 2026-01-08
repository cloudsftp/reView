use super::config::*;

use std::{io::Read as _, net::TcpStream, thread::sleep, time::Duration};

use anyhow::{Context, Error};
use gstreamer_app::AppSrc;
use lz4_flex::frame::FrameDecoder;
use tracing::info;

use gstreamer::{Pipeline, prelude::*};

pub async fn gstreamer_thread() -> Result<(), Error> {
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
