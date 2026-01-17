use super::config::*;

use std::{io::Read as _, thread::sleep, time::Duration};

use anyhow::{Context, Error};
use futures::stream::StreamExt;
use gstreamer_app::AppSrc;
use gstreamer_video::VideoFormat;
use lz4_flex::frame::FrameDecoder;
use review_server::config::{CommunicatedConfig, PixelFormat};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info, trace};

use gstreamer::{Pipeline, prelude::*};

pub async fn gstreamer_thread(opts: ClientOptions) -> Result<(), Error> {
    gstreamer::init().context("could not init gstreamer")?;

    sleep(Duration::from_millis(100));

    info!("setting up TCP connection");
    let stream = TcpStream::connect(format!("{}:{}", opts.remarkable_ip, opts.tcp_port))
        .await
        .context("could not connect to TCP stream")?;

    let mut framed_stream = Framed::new(stream, LengthDelimitedCodec::new());
    let communicated_config = get_communicated_config(&mut framed_stream)
        .await
        .context("could not get communicated config from TCP stream")?;

    debug!("received communicated config: {:?}", &communicated_config);

    /*
    let mut stream = stream
        .into_std()
        .context("could not convert stream into std")?;

    // let mut decoded_video_data = FrameDecoder::new(stream);

    */

    let (pipeline, appsrc) =
        build_pipeline(&communicated_config).context("could not build gstreamer pipeline")?;
    pipeline
        .set_state(gstreamer::State::Playing)
        .context("could not start playing gstreamer pipeline")?;

    let n = 4;

    loop {
        let mut full_frame = vec![];
        for _ in 0..n {
            debug!("attempting to read data from TCP stream");

            let frame = framed_stream
                .next()
                .await
                .context("TCP stream was closed")?
                .context("could not read from TCP stream")?;

            debug!("received {} bytes from TCP stream", frame.len());

            full_frame.append(&mut frame.iter().copied().collect());
        }

        let buffer = gstreamer::Buffer::from_slice(full_frame);
        appsrc
            .push_buffer(buffer)
            .context("could not push buffer to app source")?;
    }
}

async fn get_communicated_config(
    framed_stream: &mut Framed<TcpStream, LengthDelimitedCodec>,
) -> Result<CommunicatedConfig, Error> {
    let config_bytes = framed_stream
        .next()
        .await
        .context("received None as config bytes")?
        .context("could not receive config bytes")?;

    let config = bson::deserialize_from_slice(&config_bytes)
        .context("could not deserialize config from bytes")?;

    Ok(config)
}

fn to_video_format(pixel_format: &PixelFormat) -> VideoFormat {
    match pixel_format {
        PixelFormat::Rgb565le => todo!("not sure what the video format for RGB 565 LE is"),
        PixelFormat::Gray8 => VideoFormat::Gray8,
        PixelFormat::Gray16be => VideoFormat::Gray16Be,
        PixelFormat::Bgra => VideoFormat::Bgra,
    }
}

fn build_pipeline(communicated_config: &CommunicatedConfig) -> Result<(Pipeline, AppSrc), Error> {
    let video_info = gstreamer_video::VideoInfo::builder(
        to_video_format(&communicated_config.video_config.pixel_format),
        communicated_config.video_config.width as u32,
        communicated_config.video_config.height as u32,
    )
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
