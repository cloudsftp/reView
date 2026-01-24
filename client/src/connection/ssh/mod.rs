use std::{fs::read_to_string, path::PathBuf};

use anyhow::{Context, Error};
use ssh_key::PrivateKey;
use tracing::debug;

use crate::config::ClientOptions;

use super::Connection;

impl Connection {
    pub async fn authenticate(&mut self, client_options: ClientOptions) -> Result<(), Error> {
        let key = get_ssh_key(&client_options.ssh_key).context("could not get private SSH key")?;
        let pub_key = key.public_key();

        self.send(pub_key)
            .await
            .context("could not send public SSH key")?;

        Ok(())
    }
}

fn get_ssh_key(key_file_path: &PathBuf) -> Result<PrivateKey, Error> {
    let content = read_to_string(key_file_path).context("could not read private SSH key file")?;
    let key = PrivateKey::from_openssh(content)?;

    debug!("read private key: {:?}", key);

    Ok(key)
}
