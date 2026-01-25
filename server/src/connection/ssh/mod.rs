use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Error};
use ssh_encoding::{Decode, Reader};
use ssh_key::{AuthorizedKeys, PublicKey, SshSig, authorized_keys::Entry};
use tracing::debug;

use super::Connection;

impl Connection {
    pub async fn authenticate(&mut self) -> Result<PublicKey, Error> {
        let pub_key = self
            .find_authorized_key()
            .await
            .context("could not find an authorized key")?;

        debug!("received public SSH key: {:?}", pub_key);

        self.request_signature(&pub_key)
            .await
            .context("could not get signature from client")?;

        Ok(pub_key)
    }

    async fn find_authorized_key(&mut self) -> Result<PublicKey, Error> {
        let authorized_keys = get_authorized_keys().context("could not get authorized keys")?;

        let pub_keys: Vec<PublicKey> = self
            .receive()
            .await
            .context("could not receive public SSH keys to check for authorization")?;

        let authorized_key_index = pub_keys
            .iter()
            .enumerate()
            .filter_map(|(index, pub_key)| {
                is_key_authorized(pub_key, &authorized_keys).then_some(index)
            })
            .next()
            .context("none of the provided public keys is authorized")?;

        self.send(&authorized_key_index)
            .await
            .context("could not send index of authorized key")?;

        Ok(pub_keys[authorized_key_index].clone())
    }

    async fn request_signature(&mut self, pub_key: &PublicKey) -> Result<(), Error> {
        let message: [u8; 128] = rand::random();
        self.send_raw(message.to_vec().into())
            .await
            .context("could not send message to sign")?;

        let encoded_signature = self
            .receive_raw()
            .await
            .context("culd not get signed message back")?
            .to_vec();
        let mut encoded_signature = encoded_signature.as_slice();
        let signature =
            SshSig::decode(&mut encoded_signature).context("could not decode signature")?;

        pub_key
            .verify("review", &message, &signature)
            .context("could not verify signature")?;

        Ok(())
    }
}

fn is_key_authorized(pub_key: &PublicKey, authorized_keys: &Vec<Entry>) -> bool {
    authorized_keys
        .iter()
        .any(|authorized_key| authorized_key.public_key() == pub_key)
}

fn get_authorized_keys() -> Result<Vec<Entry>, Error> {
    AuthorizedKeys::read_file(
        PathBuf::from_str("/home/root/.ssh/authorized_keys")
            .context("could not build path of authorized keys")?,
    )
    .context("could not read authorized keys file")
}
