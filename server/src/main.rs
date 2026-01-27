mod config;
mod connection;
mod framebuffer;
mod version;

use std::fs::OpenOptions;

use anyhow::{Context, Error};
use clap::Parser;
use config::CliOptions;
use tokio::{net::TcpListener, spawn};
use tracing::{error, info};
use tracing_subscriber::{Registry, fmt, layer::SubscriberExt};

use connection::{Connection, video::VideoConnection};

#[tokio::main]
async fn main() -> Result<(), Error> {
    initialize_logging().context("could not initialize logging")?;

    let cli_options = CliOptions::parse();

    info!("setting up TCP server");

    let mut server = Server::new(cli_options)
        .await
        .context("could not create new server")?;

    server.run().await.context("TCP server stopped")?;

    Ok(())
}

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
}

impl Server {
    async fn new(cli_options: CliOptions) -> Result<Self, Error> {
        let listener = TcpListener::bind(&format!("0.0.0.0:{}", cli_options.port))
            .await
            .context(format!("could not bind to port {}", cli_options.port))?;

        Ok(Self { listener })
    }

    async fn run(&mut self) -> Result<(), Error> {
        loop {
            let (stream, addr) = self
                .listener
                .accept()
                .await
                .context("error while waiting for a TCP connection")?;

            stream.set_nodelay(true).context("could not set nodelay")?;

            info!("new connection from {}", addr);

            let conn = Connection::new(stream);
            spawn(async move {
                if let Err(error) = Self::task(conn).await {
                    error!("connection terminated with error {:?}", error);
                }
            });
        }
    }

    async fn task(mut conn: Connection) -> Result<(), Error> {
        let stream_config = conn
            .initialize_communication()
            .await
            .context("error during initializing the communication")?;

        let mut video_conn = VideoConnection::new(conn, stream_config)
            .context("could not initialize video connection")?;
        video_conn.run().await.context("error while streaming")?;

        Ok(())
    }
}

fn initialize_logging() -> Result<(), Error> {
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./review.log")
        .context("could not open log file")?;

    let subscriber = Registry::default()
        .with(fmt::layer().with_writer(log_file))
        .with(fmt::layer().with_ansi(true).compact());

    tracing::subscriber::set_global_default(subscriber)
        .context("could not set global subscriber")?;

    Ok(())
}
