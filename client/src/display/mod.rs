use super::config::*;

use std::{io::Read as _, thread::sleep, time::Duration};

use anyhow::{Context, Error};
use gstreamer_app::AppSrc;
use gstreamer_video::VideoFormat;
use review_server::config::device::{PixelFormat, VideoConfig};
use tracing::{debug, info};

use gstreamer::{Pipeline, prelude::*};

#[derive(Debug)]
pub struct Display {
    pipeline: Pipeline,
    appsrc: AppSrc,
}

impl Display {
    pub fn new(video_config: VideoConfig) -> Result<Self, Error> {
        gstreamer::init().context("could not init gstreamer")?;

        let (pipeline, appsrc) =
            build_pipeline(&video_config).context("could not build gstreamer pipeline")?;
        pipeline
            .set_state(gstreamer::State::Playing)
            .context("could not start playing gstreamer pipeline")?;

        Ok(Self { pipeline, appsrc })
    }

    pub fn push_frame(&mut self, frame: Vec<u8>) -> Result<(), Error> {
        let buffer = gstreamer::Buffer::from_mut_slice(frame);
        self.appsrc
            .push_buffer(buffer)
            .context("could not push buffer to app source")
            .map(|_| ())
    }
}

fn build_pipeline(video_config: &VideoConfig) -> Result<(Pipeline, AppSrc), Error> {
    let video_info = gstreamer_video::VideoInfo::builder(
        to_video_format(&video_config.pixel_format),
        video_config.width as u32,
        video_config.height as u32,
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

fn to_video_format(pixel_format: &PixelFormat) -> VideoFormat {
    match pixel_format {
        PixelFormat::Rgb565le => todo!("not sure what the video format for RGB 565 LE is"),
        PixelFormat::Gray8 => VideoFormat::Gray8,
        PixelFormat::Gray16be => VideoFormat::Gray16Be,
        PixelFormat::Bgra => VideoFormat::Bgra,
    }
}
