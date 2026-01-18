/*
use super::config::*;

use std::{io::Read as _, thread::sleep, time::Duration};

use anyhow::{Context, Error};
use futures::stream::StreamExt;
use gstreamer_app::AppSrc;
use gstreamer_video::VideoFormat;
use lz4_flex::decompress_size_prepended;
use review_server::config::{CommunicatedConfig, PixelFormat};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info};

use gstreamer::{Pipeline, prelude::*};

pub async fn gstreamer_thread(opts: ClientOptions) -> Result<(), Error> {
    gstreamer::init().context("could not init gstreamer")?;

    let (pipeline, appsrc) =
        build_pipeline(&communicated_config).context("could not build gstreamer pipeline")?;
    pipeline
        .set_state(gstreamer::State::Playing)
        .context("could not start playing gstreamer pipeline")?;

    loop {
        debug!("attempting to read data from TCP stream");

        let compressed_frame = framed_stream
            .next()
            .await
            .context("TCP stream was closed")?
            .context("could not read from TCP stream")?;

        debug!(
            "read one compressed frame from TCP stream ({} bytes)",
            compressed_frame.len(),
        );

        let frame = decompress_size_prepended(&compressed_frame)
            .context("could not decompress received frame")?;

        debug!("decompressed: {} bytes", frame.len());

        let buffer = gstreamer::Buffer::from_mut_slice(frame);
        appsrc
            .push_buffer(buffer)
            .context("could not push buffer to app source")?;
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
*/
