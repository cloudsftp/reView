use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Error};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use ssh_encoding::{Decode, Encode};
use ssh_key::{AuthorizedKeys, PublicKey, SshSig, authorized_keys::Entry};

use super::Connection;

pub const SIGNATURE_NAMESPACE: &str = "review";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthentificationToken {
    pub token: Vec<u8>,
}

const TOKEN_SIZE: usize = 256;

impl AuthentificationToken {
    fn new() -> Self {
        let mut rng = StdRng::from_os_rng();
        let token: [u8; TOKEN_SIZE] = rng.random();
        let token = token.to_vec();

        Self { token }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signatures {
    pub encoded_signatures: Vec<Vec<u8>>,
}

impl TryFrom<Vec<SshSig>> for Signatures {
    type Error = Error;

    fn try_from(signatures: Vec<SshSig>) -> Result<Self, Self::Error> {
        let encoded_signatures = signatures
            .iter()
            .map(|signature| -> Result<_, Error> {
                let mut encoded_signature = vec![];
                signature
                    .encode(&mut encoded_signature)
                    .context("could not encode signature")?;

                Ok(encoded_signature)
            })
            .collect::<Result<_, Error>>()?;

        Ok(Self { encoded_signatures })
    }
}

impl TryInto<Vec<SshSig>> for Signatures {
    type Error = Error;

    fn try_into(self) -> Result<Vec<SshSig>, Self::Error> {
        self.encoded_signatures
            .iter()
            .map(|signature_bytes| {
                let mut signature_bytes = signature_bytes.as_slice();
                SshSig::decode(&mut signature_bytes).context("could not decode signature")
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizedPublicKey {
    pub index: usize,
}

impl Connection {
    pub async fn authenticate(&mut self) -> Result<PublicKey, Error> {
        let token = AuthentificationToken::new();
        self.send(&token)
            .await
            .context("could not send authentification token to client")?;

        let pub_key = self
            .find_authorized_key(&token)
            .await
            .context("could not find an authorized key")?;

        Ok(pub_key)
    }

    async fn find_authorized_key(
        &mut self,
        token: &AuthentificationToken,
    ) -> Result<PublicKey, Error> {
        let authorized_keys = get_authorized_keys().context("could not get authorized keys")?;

        let signatures: Signatures = self
            .receive()
            .await
            .context("could not receive public keys")?;
        let signatures: Vec<SshSig> = signatures.try_into()?;

        let (authorized_key_index, pub_key) = signatures
            .iter()
            .enumerate()
            .filter_map(|(index, signature)| {
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
    token: &AuthentificationToken,
    signature: &SshSig,
    authorized_keys: &[Entry],
) -> Option<PublicKey> {
    authorized_keys
        .iter()
        .filter_map(|entry| {
            let pub_key = entry.public_key();
            pub_key
                .verify(SIGNATURE_NAMESPACE, &token.token, signature)
                .ok()
                .map(|_| pub_key)
        })
        .next()
        .cloned()
}

fn get_authorized_keys() -> Result<Vec<Entry>, Error> {
    AuthorizedKeys::read_file(
        PathBuf::from_str("/home/root/.ssh/authorized_keys")
            .context("could not build path of authorized keys")?,
    )
    .context("could not read authorized keys file")
}
