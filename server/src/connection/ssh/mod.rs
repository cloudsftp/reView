use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use ssh_encoding::{Decode, Encode};
use ssh_key::{AuthorizedKeys, PublicKey, SshSig, authorized_keys::Entry};
use tracing::debug;

use super::Connection;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthentificationToken {
    pub token: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeys {
    pub keys_and_signatures: Vec<PublicKeyAndSignature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeyAndSignature {
    key: Vec<u8>,
    signature: Vec<u8>,
}

impl TryFrom<&(PublicKey, SshSig)> for PublicKeyAndSignature {
    type Error = Error;

    fn try_from((pub_key, signature): &(PublicKey, SshSig)) -> Result<Self, Self::Error> {
        let encoded_key = pub_key.to_bytes().context(format!(
            "could not encode public key '{}'",
            pub_key.comment(),
        ))?;

        let mut encoded_signature = vec![];
        signature
            .encode(&mut encoded_signature)
            .context("could not encode signature")?;

        Ok(PublicKeyAndSignature {
            key: encoded_key,
            signature: encoded_signature,
        })
    }
}

impl TryInto<(PublicKey, SshSig)> for &PublicKeyAndSignature {
    type Error = Error;

    fn try_into(self) -> Result<(PublicKey, SshSig), Self::Error> {
        let pub_key = PublicKey::from_bytes(&self.key)
            .context("could not decode public key from ssh string")?;

        let mut signature_bytes = self.signature.as_slice();
        let signature =
            SshSig::decode(&mut signature_bytes).context("could not decode signature")?;

        Ok((pub_key, signature))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizedPublicKey {
    pub index: usize,
}

impl Connection {
    pub async fn authenticate(&mut self) -> Result<PublicKey, Error> {
        let token: [u8; 128] = rand::random();
        let token = token.to_vec();
        let token_message = AuthentificationToken {
            token: token.clone(),
        };
        self.send(&token_message)
            .await
            .context("could not send authentification token to client")?;

        let pub_key = self
            .find_authorized_key(&token)
            .await
            .context("could not find an authorized key")?;

        Ok(pub_key)
    }

    async fn find_authorized_key(&mut self, token: &Vec<u8>) -> Result<PublicKey, Error> {
        let authorized_keys = get_authorized_keys().context("could not get authorized keys")?;

        let pub_keys: PublicKeys = self
            .receive()
            .await
            .context("could not receive public keys")?;

        let keys_and_signatures = pub_keys
            .keys_and_signatures
            .iter()
            .map(|key_and_signature| key_and_signature.try_into())
            .collect::<Result<Vec<_>, Error>>()
            .context("could not parse public keys and signatures")?;

        let (authorized_key_index, pub_key) = keys_and_signatures
            .iter()
            .enumerate()
            .filter_map(|(index, (_, signature))| {
                get_authorized_key_matching_signature(token, signature, &authorized_keys)
                    .map(|pub_key| (index, pub_key))
            })
            .next()
            .context("none of the provided public keys is authorized")?;

        let authorized_key_message = AuthorizedPublicKey {
            index: authorized_key_index,
        };

        self.send(&authorized_key_message)
            .await
            .context("could not send index of authorized key")?;

        Ok(pub_key)
    }
}

fn get_authorized_key_matching_signature(
    token: &Vec<u8>,
    signature: &SshSig,
    authorized_keys: &Vec<Entry>,
) -> Option<PublicKey> {
    authorized_keys
        .iter()
        .filter_map(|entry| {
            let pub_key = entry.public_key();
            debug!("public key: {:?}, signature: {:?}", pub_key, signature);
            debug!("token (length: {}), {:?}", token.len(), token);
            pub_key
                .verify("review", token, signature)
                .ok()
                .map(|_| pub_key)
        })
        .cloned()
        .next()
}

fn get_authorized_keys() -> Result<Vec<Entry>, Error> {
    AuthorizedKeys::read_file(
        PathBuf::from_str("/home/root/.ssh/authorized_keys")
            .context("could not build path of authorized keys")?,
    )
    .context("could not read authorized keys file")
}
