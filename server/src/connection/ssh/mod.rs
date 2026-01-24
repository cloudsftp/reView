use anyhow::{Context, Error};
use ssh_key::PublicKey;
use tracing::debug;

use super::Connection;

impl Connection {
    pub async fn authenticate(&mut self) -> Result<(), Error> {
        let pub_key: PublicKey = self
            .receive()
            .await
            .context("could not receive public SSH key")?;

        debug!("received public SSH key: {:?}", pub_key);

        Ok(())
    }
}
