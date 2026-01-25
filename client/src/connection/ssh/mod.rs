mod keys;

use anyhow::{Context, Error};
use itertools::Itertools;
use ssh_encoding::Encode;
use ssh_key::{PrivateKey, PublicKey};
use tokio_util::bytes::BytesMut;
use tracing::info;

use super::Connection;
use crate::config::ClientOptions;
use keys::get_keys_to_check;

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

        self.sign_request(&priv_key)
            .await
            .context("could not sign the requested message")?;

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

    async fn sign_request(&mut self, priv_key: &PrivateKey) -> Result<(), Error> {
        let message = self
            .receive_raw()
            .await
            .context("could not receive message to sign")?
            .to_vec();

        let signature = priv_key
            .sign("review", ssh_key::HashAlg::Sha512, &message)
            .context("could not sign the requested message")?;

        let mut encoded_signature = vec![];
        signature
            .encode(&mut encoded_signature)
            .context("could not encode signature")?;
        self.send_raw(encoded_signature.into())
            .await
            .context("could not send out encoded signature")?;

        Ok(())
    }
}
