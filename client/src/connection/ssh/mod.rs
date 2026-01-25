use std::{env::home_dir, fs::read_to_string, path::PathBuf};

use anyhow::{Context, Error, anyhow};
use ssh_key::PrivateKey;
use tracing::{debug, info, trace};

use crate::config::ClientOptions;

use super::Connection;

impl Connection {
    pub async fn authenticate(&mut self, client_options: ClientOptions) -> Result<(), Error> {
        let keys_to_check = get_keys_to_check(&client_options.ssh_key)
            .context("could not get private keys to check")?;

        info!("found {} private SSH keys to check", keys_to_check.len());

        /*
        let key = get_ssh_key(&client_options.ssh_key).context("could not get private SSH key")?;
        let pub_key = key.public_key();

        self.send(pub_key)
            .await
            .context("could not send public SSH key")?;
        */

        Ok(())
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
            trace!("attemting to parse {} as a private SSH key", name);
            let private_key = load_private_key(&ssh_directory.join(name)).ok()?;
            trace!("successfully parsed {} as a private SSH key", name);

            Some(private_key)
        })
        .collect())
}

fn load_private_key(path: &PathBuf) -> Result<PrivateKey, Error> {
    let content = read_to_string(path).context("could not read private SSH key file")?;
    let key = PrivateKey::from_openssh(content)?;

    Ok(key)
}

/*
fn get_ssh_key(key_file_path: &PathBuf) -> Result<PrivateKey, Error> {
    let content = read_to_string(key_file_path).context("could not read private SSH key file")?;
    let key = PrivateKey::from_openssh(content)?;

    debug!("read private key: {:?}", key);

    Ok(key)
}
*/
