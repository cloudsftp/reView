use std::path::PathBuf;

use version::ConfigVersion;

pub mod version;

#[derive(Debug)]
pub enum VideoDataSource {
    File { path: PathBuf },
    Memory { skip: usize },
}

pub struct Config {
    pub height: usize,
    pub width: usize,
    pub bytes_per_pixel: usize,
    pub source: VideoDataSource,
}

pub fn get_config(config_version: &ConfigVersion) -> Config {
    todo!()
}
