mod keys;

use anyhow::{Context, Error, anyhow};
use itertools::Itertools;
use review_server::connection::ssh::{
    AuthentificationToken, AuthorizedPublicKey, SIGNATURE_NAMESPACE, Signatures,
};
use ssh_key::{HashAlg, PrivateKey};
use tracing::{info, warn};

use super::Connection;
use crate::config::ClientOptions;
use keys::get_keys_to_check;

impl Connection {
    pub async fn authenticate(&mut self, client_options: ClientOptions) -> Result<(), Error> {
        let token: AuthentificationToken = self
            .receive()
            .await
            .context("could not receive authentification token")?;

        let keys_to_check = get_keys_to_check(&client_options.ssh_key)
            .context("could not get private keys to check")?;
        info!("found {} private SSH keys to check", keys_to_check.len());

        let priv_key = self
            .find_authorized_key(keys_to_check, token.token)
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
        token: Vec<u8>,
    ) -> Result<PrivateKey, Error> {
        let signatures = keys_to_check
            .iter()
            .filter_map(|priv_key| {
                match priv_key.sign(SIGNATURE_NAMESPACE, HashAlg::Sha512, &token) {
                    Ok(signature) => Some(signature),
                    Err(err) => {
                        warn!(
                            "private key '{}' could not sign authentification token: {:?}",
                            priv_key.comment(),
                            err,
                        );
                        None
                    }
                }
            })
            .collect_vec();

        if signatures.is_empty() {
            return Err(anyhow!(
                "none of the private SSH keys could sign the authentification token"
            ));
        }

        let signatures: Signatures = signatures.try_into()?;

        self.send(&signatures)
            .await
            .context("could not send over all public keys to check")?;

        let authorized_key: AuthorizedPublicKey = self
            .receive()
            .await
            .context("could not receive authorized key index")?;

        Ok(keys_to_check[authorized_key.index].clone())
    }
}
