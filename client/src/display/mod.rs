use super::config::*;

use std::{io::Read as _, thread::sleep, time::Duration};

use anyhow::{Context, Error};
use futures::stream::StreamExt;
use gstreamer_app::AppSrc;
use lz4_flex::frame::FrameDecoder;
use review_server::config::CommunicatedConfig;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info};

use gstreamer::{Pipeline, prelude::*};

pub async fn gstreamer_thread(opts: ClientOptions) -> Result<(), Error> {
    gstreamer::init().context("could not init gstreamer")?;

    sleep(Duration::from_millis(100));

    info!("setting up TCP connection");
    let stream = TcpStream::connect(format!("{}:{}", opts.remarkable_ip, opts.tcp_port))
        .await
        .context("could not connect to TCP stream")?;

    let (stream, communicated_config) = get_communicated_config(stream)
        .await
        .context("could not get communicated config from TCP stream")?;

    debug!("received communicated config: {:?}", &communicated_config);

    let stream = stream
        .into_std()
        .context("could not convert stream into std")?;

    let mut decoded_video_data = FrameDecoder::new(stream);

    sleep(Duration::from_secs(1));

    let (pipeline, appsrc) = build_pipeline().context("could not build gstreamer pipeline")?;
    pipeline
        .set_state(gstreamer::State::Playing)
        .context("could not start playing gstreamer pipeline")?;

    let mut buffer = vec![0u8; (BYTES_PER_PIXEL * HEIGHT * WIDTH) as usize];
    loop {
        let n = decoded_video_data
            .read(&mut buffer)
            .context("could not read from TCP stream")?;

        let slice = buffer[..n].to_vec();

        debug!("read {} bytes:\n\n{:?}", n, &slice);

        let buffer = gstreamer::Buffer::from_slice(slice);
        appsrc
            .push_buffer(buffer)
            .context("could not push buffer to app source")?;

        sleep(Duration::from_secs_f64(1.));
    }
}

async fn get_communicated_config(
    tcp_stream: TcpStream,
) -> Result<(TcpStream, CommunicatedConfig), Error> {
    let mut framed_stream = Framed::new(tcp_stream, LengthDelimitedCodec::new());

    let config_bytes = framed_stream
        .next()
        .await
        .context("received None as config bytes")?
        .context("could not receive config bytes")?;

    let config = bson::deserialize_from_slice(&config_bytes)
        .context("could not deserialize config from bytes")?;

    Ok((framed_stream.into_inner(), config))
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
