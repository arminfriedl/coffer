use coffer_common::certificate::Certificate;

use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use serde::Deserialize;
use serde_yaml;

pub fn encrypt_yaml(yaml:PathBuf, out: PathBuf, certificate: PathBuf) {
    let cert = Certificate::new_from_cbor(certificate).unwrap();

    let  
}
