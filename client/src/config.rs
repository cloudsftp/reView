use gstreamer_video::VideoFormat;

pub const DEFAULT_IP: &str = "10.11.99.1";
pub const SSH_PORT: u16 = 22;
pub const PORT: u16 = 6680;

pub const HEIGHT: u32 = 1872;
pub const WIDTH: u32 = 1404;
pub const PIXEL_FORMAT: &str = "bgra";
pub const VIDEO_FORMAT: VideoFormat = VideoFormat::Bgra;
pub const BYTES_PER_PIXEL: u32 = 4;
pub const FILE: &str = ":mem:";
pub const SKIP_OFFSET: usize = 2629636;

pub const APP_SOURCE_NAME: &str = "binsource";
