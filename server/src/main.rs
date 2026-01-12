mod config;
mod version;

use anyhow::{Context, Error};
use clap::Parser;
use config::{CliOptions, CommunicatedConfig, ServerOptions, VersionInfo, get_video_config};
use futures::sink::SinkExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info};
use version::{get_firmware_version, get_hardware_version};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let opts = CliOptions::parse();
    debug!("cli options: {:?}", opts);
    let opts = ServerOptions::try_from(opts).context("could not get server options")?;
    debug!("resolved options: {:?}", opts);

    let hardware_version = get_hardware_version().context("could not get hardware version")?;
    info!("Detected hardware version {:?}", hardware_version);

    let firmware_version = get_firmware_version().context("could not get version")?;
    info!("Detected firmware version {:?}", firmware_version);

    let video_config =
        get_video_config(&hardware_version, &firmware_version).context("could not get config")?;
    info!("using video config: {:?}", video_config);

    let communicated_config = CommunicatedConfig {
        version: VersionInfo {
            hardware: hardware_version,
            firmware: firmware_version,
        },
        video_config: video_config.shared,
    };

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", opts.port))
        .await
        .context(format!("could not bind to port {}", opts.port))?;

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("new connection from {}", addr);

        tokio::spawn(open_connection(
            stream,
            opts.clone(),
            communicated_config.clone(),
        ));
    }
}

async fn open_connection(
    mut stream: TcpStream,
    opts: ServerOptions,
    communicated_config: CommunicatedConfig,
) -> Result<(), Error> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
    framed
        .send(
            serde_json::to_string(&communicated_config)
                .context("could not serialize communicated config")?
                .into(),
        )
        .await
        .context("could not send out config")?;

    Ok(())
}
