use coffer_common::certificate::Certificate;

use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

pub fn generate_key(out: PathBuf) {
    let certificate = Certificate::new().unwrap();

    let cert = certificate.to_cbor().unwrap();

    let mut writer = File::create(&out)
        .expect(&format!{"Could not create out file {}", &out.display()});

    writer.write_all(&cert).unwrap();

}
