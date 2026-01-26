mod keys;

use anyhow::{Context, Error, anyhow};
use itertools::Itertools;
use review_server::connection::ssh::{
    AuthentificationToken, AuthorizedPublicKey, PublicKeyAndSignature, PublicKeys,
};
use ssh_key::{HashAlg, PrivateKey};
use tracing::{debug, error, info, warn};

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
        let pub_keys_to_check = keys_to_check
            .iter()
            .filter_map(|priv_key| {
                let pub_key = priv_key.public_key().clone();
                let signature = match priv_key.sign("review", HashAlg::Sha512, &token) {
                    Ok(signature) => signature,
                    Err(err) => {
                        warn!(
                            "private key '{}' could not sign authentification token: {:?}",
                            priv_key.comment(),
                            err,
                        );
                        return None;
                    }
                };

                debug!("token (length: {}), {:?}", token.len(), token);
                if let Err(err) = pub_key.verify("review", &token, &signature) {
                    error!("problem verifying signature: {:?}", err);
                }
                debug!("public key {} successfully signed token", pub_key.comment());

                Some((pub_key, signature))
            })
            .collect_vec();

        if pub_keys_to_check.len() == 0 {
            return Err(anyhow!(
                "none of the private SSH keys could sign the authentification token"
            ));
        }

        let pub_keys_to_check = PublicKeys {
            keys_and_signatures: pub_keys_to_check
                .iter()
                .map(PublicKeyAndSignature::try_from)
                .collect::<Result<_, Error>>()
                .context("could not encode the public keys and signatures")?,
        };

        self.send(&pub_keys_to_check)
            .await
            .context("could not send over all public keys to check")?;

        let authorized_key: AuthorizedPublicKey = self
            .receive()
            .await
            .context("could not receive authorized key index")?;

        Ok(keys_to_check[authorized_key.index].clone())
    }
}
