use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Error, anyhow};
use ssh_key::{AuthorizedKeys, PublicKey, authorized_keys::Entry};
use tracing::debug;

use super::Connection;

impl Connection {
    pub async fn authenticate(&mut self) -> Result<(), Error> {
        let authorized_keys = get_authorized_keys().context("could not get authorized keys")?;

        let pub_key: PublicKey = self
            .receive()
            .await
            .context("could not receive public SSH key")?;

        let pub_key_authorized = authorized_keys
            .iter()
            .any(|authorized_key| authorized_key.public_key() == &pub_key);

        if !pub_key_authorized {
            return Err(anyhow!("public key is not authorized"));
        }

        debug!("received public SSH key: {:?}", pub_key);

        Ok(())
    }
}

fn get_authorized_keys<'a>() -> Result<Vec<Entry>, Error> {
    AuthorizedKeys::read_file(
        PathBuf::from_str("/home/root/.ssh/authorized_keys")
            .context("could not build path of authorized keys")?,
    )
    .context("could not read authorized keys file")
}
