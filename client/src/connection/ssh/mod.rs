use std::{env::home_dir, fs::read_to_string, path::PathBuf};

use anyhow::{Context, Error, anyhow};
use itertools::Itertools;
use ssh_key::{PrivateKey, PublicKey};
use tracing::{debug, info, trace};

use super::Connection;
use crate::config::ClientOptions;

impl Connection {
    pub async fn authenticate(&mut self, client_options: ClientOptions) -> Result<(), Error> {
        let keys_to_check = get_keys_to_check(&client_options.ssh_key)
            .context("could not get private keys to check")?;

        info!("found {} private SSH keys to check", keys_to_check.len());

        let priv_key = self
            .find_authorized_key(keys_to_check)
            .await
            .context("could not find an authorized private key")?;

        info!(
            "the private SSH key '{}' with algorithm '{:?}' is authorized",
            priv_key.comment(),
            priv_key.algorithm(),
        );

        Ok(())
    }

    async fn find_authorized_key(
        &mut self,
        keys_to_check: Vec<PrivateKey>,
    ) -> Result<PrivateKey, Error> {
        let pub_keys_to_check = keys_to_check
            .iter()
            .map(|priv_key| priv_key.public_key())
            .cloned()
            .collect_vec();

        self.send(&pub_keys_to_check)
            .await
            .context("could not send over all public keys to check")?;

        let authorized_key_index: usize = self
            .receive()
            .await
            .context("could not receive authorized key index")?;

        Ok(keys_to_check[authorized_key_index].clone())
    }
}

fn get_keys_to_check(private_key_path: &Option<PathBuf>) -> Result<Vec<PrivateKey>, Error> {
    if let Some(private_key_path) = private_key_path {
        debug!(
            "private key at {} explicitly defined, only loading this key",
            private_key_path.to_string_lossy()
        );
        return Ok(vec![
            load_private_key(private_key_path)
                .context("could not load explicitly specified private SSH key")?,
        ]);
    }
    debug!("no private key explicitly defined, loading keys from .ssh directory");

    let ssh_directory = home_dir()
        .context("could not get home directory")?
        .join(".ssh");

    if !ssh_directory.exists() || ssh_directory.is_file() {
        return Err(anyhow!(
            "SSH directory '{}' does not exist or is a file",
            ssh_directory.to_string_lossy()
        ));
    }

    let dir_entries = ssh_directory
        .read_dir()
        .context("could not read the SSH directory")?;

    Ok(dir_entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let path = ssh_directory.join(name);

            trace!(
                "attemting to parse {} as a private SSH key",
                path.to_string_lossy()
            );
            let private_key = load_private_key(&path).ok()?;
            trace!(
                "successfully parsed {} as a private SSH key",
                path.to_string_lossy()
            );

            Some(private_key)
        })
        .collect())
}

fn load_private_key(path: &PathBuf) -> Result<PrivateKey, Error> {
    let content = read_to_string(path).context(format!(
        "could not read private SSH key file {}",
        path.to_string_lossy()
    ))?;
    let key = PrivateKey::from_openssh(content).context(format!(
        "could not parse private key from file {}",
        path.to_string_lossy()
    ))?;

    Ok(key)
}
