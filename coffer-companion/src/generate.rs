use sodiumoxide::crypto::box_;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
pub struct MasterKey {
    pub public_key: box_::PublicKey,
    secret_key: box_::SecretKey,
}

pub fn generate_key(out: PathBuf) {
    let keypair = box_::gen_keypair();
    let masterkey: MasterKey = MasterKey {
        public_key: keypair.0,
        secret_key: keypair.1,
    };

    let writer = File::create(&out)
        .expect(&format!{"Could not create out file {}", &out.display()});

    serde_cbor::to_writer(writer, &masterkey)
        .expect(&format!{"Couldn't deserialize to key file {}", &out.display()});
}

impl From<&PathBuf> for MasterKey {
    fn from(masterkey: &PathBuf) -> Self {
        let reader = File::open(masterkey)
            .expect(&format!{"Could not read file {}", masterkey.display()});

        serde_cbor::from_reader(reader).unwrap()
    }
}
