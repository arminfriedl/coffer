#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::collections::HashMap;

use quick_error::quick_error;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sealedbox;

use crate::certificate::{Certificate, CertificateError};

quick_error! {
    #[derive(Debug)]
    pub enum KeyringError {
        UnkownClientKey
        InvalidClientKey
        Certificate(err: CertificateError) {
            from()
        }
        Msg(err: &'static str) {
            from(err)
            display("{}", err)
        }
        Other(err: Box<dyn std::error::Error>) {
            cause(&**err)
        }
    }
}

pub struct Keyring {
    certificate: Certificate,
    known_keys: HashMap<Vec<u8>, box_::PublicKey>
}

impl Keyring {
    pub fn new(certificate: Certificate) -> Keyring {
        Keyring {
            certificate: certificate,
            known_keys: HashMap::new()
        }
    }

    pub fn add_known_key(&mut self, key: &[u8]) -> Result<(), KeyringError> {
        let public_key = box_::PublicKey::from_slice(key)
            .ok_or(KeyringError::InvalidClientKey)?;

        self.known_keys.insert(Vec::from(key), public_key);
        Ok(())
    }

    pub fn open(&self, message: &[u8]) -> Result<Vec<u8>, KeyringError> {
        self.certificate.open(message)
            .map_err(|e| KeyringError::from(e))
    }

    pub fn seal(&self, client: &[u8], message: &[u8]) -> Result<Vec<u8>, KeyringError> {
        let client_key = self.known_keys.get(client)
            .ok_or(KeyringError::UnkownClientKey)?;

        Ok(sealedbox::seal(message, client_key))
    }
}
