use anyhow::Error;
use tracing::info;

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    info!("Hello from the server");

    Ok(())
}
