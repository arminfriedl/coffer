use sodiumoxide::crypto::sealedbox;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write};
use std::path::PathBuf;

use crate::generate::MasterKey;

type Secrets = HashMap<String, String>;

pub fn generate_encrypted_secrets(yaml: PathBuf, out: PathBuf, masterkey: PathBuf) {
    let reader = File::open(&yaml)
        .expect(&format!{"Could not read file {}", masterkey.display()});

    let secrets: Secrets = serde_yaml::from_reader(reader)
        .expect(&format!{"Could not deserialize secrets from {}", &yaml.display()});

    let masterkey: MasterKey = (&masterkey).into();

    let secrets_bin = serde_cbor::to_vec(&secrets).unwrap();
    let sealed_secrets_bin = sealedbox::seal(&secrets_bin, &masterkey.public_key);

    File::create(&out)
        .and_then(|mut f| f.write(&sealed_secrets_bin))
        .expect(&format!{"Could not create out file {}", &out.display()});
}
