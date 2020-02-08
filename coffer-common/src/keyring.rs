#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::path::Path;
use std::collections::HashMap;

use quick_error::quick_error;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sealedbox;

use toml::Value as TomlValue;

use crate::certificate::{Certificate, CertificateError};

quick_error! {
    #[derive(Debug)]
    pub enum KeyringError {
        UnkownClientKey
        InvalidClientKey
        Certificate(err: CertificateError) {
            from()
        }
        HexDecodeError(err: hex::FromHexError) {
            from()
        }
        IoError(err: std::io::Error) {
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
            certificate,
            known_keys: HashMap::new()
        }
    }

    pub fn new_from_path<T>(certificate_path: T) -> Keyring
    where T: AsRef<Path>
    {
        Keyring {
            certificate: Certificate::new_from_cbor(certificate_path).unwrap(),
            known_keys: HashMap::new()
        }
    }

    pub fn add_known_keys_toml(&mut self, toml: &str) -> Result<(), KeyringError> {
        // parse the string into a toml Table
        let clients: toml::value::Table = match toml.parse::<TomlValue>().unwrap() {
            TomlValue::Table(t) => t,
            _ => panic!{"Invalid secrets file"}
        };

        self.add_known_keys_toml_table(&clients)?;

        debug!{"Known keys {:?}", self.known_keys}

        Ok(())
    }

    fn add_known_keys_toml_table(&mut self, toml_table: &toml::value::Table) -> Result<(), KeyringError> {
         // table has an no id, recourse into subtables
        if toml_table.get("id").is_none() {
            debug!{"{:?}", toml_table}
            for (_key, val) in toml_table.iter() {
                match val {
                    TomlValue::Table(subtable) => {
                        self.add_known_keys_toml_table(subtable)?;
                    },
                    _ => panic!{"Invalid secrets file"}
                }
            }

            return Ok(());
        }

        let shard = toml_table.get("id").and_then(|id| id.as_str()).ok_or(KeyringError::Msg("Invalid key parsing state"))?;
        self.add_known_key(&hex::decode(shard)?)
    }

    pub fn add_known_key(&mut self, key: &[u8]) -> Result<(), KeyringError> {
        let public_key = box_::PublicKey::from_slice(key)
            .ok_or(KeyringError::InvalidClientKey)?;

        self.known_keys.insert(Vec::from(key), public_key);
        Ok(())
    }

    pub fn open(&self, message: &[u8]) -> Result<Vec<u8>, KeyringError> {
        self.certificate.open(message)
            .map_err(KeyringError::from)
    }

    pub fn seal(&self, client: &[u8], message: &[u8]) -> Result<Vec<u8>, KeyringError> {
        let client_key = self.known_keys.get(client)
            .ok_or(KeyringError::UnkownClientKey)?;

        Ok(sealedbox::seal(message, client_key))
    }
}
