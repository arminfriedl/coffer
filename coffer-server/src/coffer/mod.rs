#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use quick_error::quick_error;

use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

// do not provide a path to keyring through coffer module. keyring is hence
// effectively private outside coffer
mod keyring;
use keyring::Keyring;

quick_error! {
    #[derive(Debug)]
    pub enum CofferError {
        Keyring(err: keyring::KeyringError) {
            from()
        }
        Io(err: std::io::Error) {
            from()
        }
        Cbor(err: serde_cbor::Error) {
            from()
        }
        Coffer(err: &'static str) {
            from()
        }
    }
}

type Result<T> = std::result::Result<T, CofferError>;
type Secrets = HashMap<String, String>; // move this to a module if it gathers crust

pub struct Coffer {
    // do not expose inner structure of coffer
    keyring: Keyring,
    secrets: Secrets
}

impl Coffer {
    /// Create a coffer from a masterkey and secrets encrypted with the masterkey's
    /// public key
    pub fn new_from_path_encrypted(masterkey: &PathBuf, secrets: &PathBuf, keep: bool) -> Result<Coffer> {
        debug!{"Initializing keyring"}
        let keyring = Keyring::new_from_path(masterkey)?;

        debug!{"Loading secrets"}
        let mut sec_data = Vec::new();
        File::open(secrets)?.read_to_end(&mut sec_data)?;

        debug!{"Removing files"}
        if !keep {
            std::fs::remove_file(secrets)?;
            std::fs::remove_file(masterkey)?;
        };

        debug!{"Decrypting secrets"}
        sec_data = keyring.master.decrypt(&sec_data)?;
        let secrets = serde_cbor::from_slice::<Secrets>(&sec_data)?;

        debug!{"Filling coffer"};
        Ok(Coffer{keyring, secrets})
    }

    pub fn get_secret(&self, key: &str) -> Result<&String> {
        self.secrets.get(key).ok_or("No secret found in coffer for".into())
    }
}
