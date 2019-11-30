#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use quick_error::quick_error;

use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use sodiumoxide::crypto::sealedbox;
use sodiumoxide::crypto::box_;

quick_error! {
    #[derive(Debug)]
    pub enum KeyringError {
        Io(err: std::io::Error) {
            from()
        }
        Cbor(err: serde_cbor::Error) {
            from()
        }
        Crypto {}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MasterKey {
    public_key: box_::PublicKey,
    secret_key: box_::SecretKey,
}

impl MasterKey {
    pub fn decrypt(&self, c: &[u8]) -> Result<Vec<u8>, KeyringError> {
        sealedbox::open(c, &self.public_key, &self.secret_key)
            .map_err(|_| KeyringError::Crypto)
    }

    fn encrypt(&self, m: &[u8]) -> Vec<u8> {
        sealedbox::seal(m, &self.public_key)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientKey {
    id: String,
    public_key: box_::PublicKey,
}

pub struct Keyring {
    pub master: MasterKey,
    pub clients: HashMap<String, ClientKey>,
}

impl Keyring {
    pub fn new_from_path(path: &PathBuf) -> Result<Keyring, KeyringError> {
        let keyring = Keyring {
            master: key_from_path(path)?,
            clients: HashMap::new(),
        };

        Ok(keyring)
    }

    pub fn add_key_from_path(&mut self, path: &PathBuf, keep: bool) -> Result<(), KeyringError> {
        let client_key: ClientKey = key_from_path(path)?;
        self.clients.insert(client_key.id.clone(), client_key);

        Ok(())
    }
}

fn key_from_path<T>(path: &PathBuf) -> Result<T, KeyringError>
where T: serde::de::DeserializeOwned
{

    let mk_file = File::open(path)?;
    let key = serde_cbor::from_reader(mk_file)?;

    Ok(key)
}
