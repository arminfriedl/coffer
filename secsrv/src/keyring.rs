#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use sodiumoxide::crypto::box_;
use std::error::Error;

#[derive(Debug,Serialize,Deserialize)]
pub enum Key {
    MasterKey {
        id: String,
        public_key: box_::PublicKey,
        secret_key: box_::SecretKey
    },

    ClientKey {
        id: String,
        public_key: box_::PublicKey
    }
}

pub fn parse_from_path(path: &PathBuf, keep: bool) -> Result<Key, Box<dyn Error>> {
    let mut mk_file = File::open(path)?;

    let mut mk_data = Vec::new();
    mk_file.read_to_end(&mut mk_data)?;
    if !keep { std::fs::remove_file(path)?; };

    Ok(serde_cbor::from_slice::<Key>(&mk_data)?)
}
