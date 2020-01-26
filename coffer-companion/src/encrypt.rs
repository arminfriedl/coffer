use coffer_common::certificate::Certificate;

use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::io::Write;

#[allow(unused)]
pub fn encrypt_yaml(yaml:PathBuf, out: PathBuf, certificate: PathBuf) {
    let cert = Certificate::new_from_cbor(certificate).unwrap();
    let mut secrets = Vec::new();
    File::open(yaml).unwrap().read_to_end(&mut secrets).unwrap();

    let sealed = cert.seal(&secrets).unwrap();
    let mut out_file = File::create(out).unwrap();
    out_file.write_all(&sealed);
}
