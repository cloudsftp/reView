use std::{env::home_dir, fs::read_to_string, path::PathBuf};

use anyhow::{Context, Error, anyhow};
use ssh_key::PrivateKey;
use tracing::{debug, trace, warn};

pub fn get_keys_to_check(private_key_path: &Option<PathBuf>) -> Result<Vec<PrivateKey>, Error> {
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
            let private_key = load_private_key(&path)
                .map_err(|err| {
                    let filename = path.file_name()?.to_string_lossy();
                    if !(path.ends_with(".pub")
                        || &filename == "known_hosts"
                        || &filename == "authorized_keys")
                    {
                        warn!(
                            "could not load private key {}: {:?}",
                            path.to_string_lossy(),
                            err,
                        )
                    };

                    Some(())
                })
                .ok()?;
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
