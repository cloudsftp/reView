use anyhow::Error;
use log::{debug, info};
use openssh::{KnownHosts, Session};

const HEIGHT: usize = 1872;
const WIDTH: usize = 1404;
const PIXEL_FORMAT: &str = "bgra";
const BYTES_PER_PIXEL: usize = 4;
const FILE: &str = ":mem:";
const SKIP_OFFSET: usize = 2629636;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();
    run_restream().await.unwrap();
}

async fn run_restream() -> Result<(), Error> {
    info!("connecting to reMarkable");
    let session = Session::connect("root@192.168.2.118", KnownHosts::Add).await?;

    let mut b = vec![0; 8];
    debug!("spawning restream");
    let mut restream = session
        .command("./restream")
        .args(&[
            "--height",
            &HEIGHT.to_string(),
            "--width",
            &WIDTH.to_string(),
            "--bytes-per-pixel",
            &BYTES_PER_PIXEL.to_string(),
            "--file",
            FILE,
            "--skip",
            &SKIP_OFFSET.to_string(),
        ])
        .spawn()
        .await?;
    debug!("spawned restream");
    let mut stdout = restream.stdout();
    match stdout {
        Some(_) => todo!(),
        None => panic!("could not get stdout"),
    }

    session.close().await?;

    Ok(())
}
