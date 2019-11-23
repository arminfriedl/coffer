use std::error::Error;
use std::fs::File;
use sodiumoxide::crypto::box_;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::io::Write;

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

pub type Secrets = HashMap<String, String>;
pub type Secs = Vec<String>;

fn main() -> Result<(), Box<dyn Error>> {
    let keypair =  box_::gen_keypair();
    let masterkey: Key = Key::MasterKey {
        id: "master".to_owned(),
        public_key: keypair.0,
        secret_key: keypair.1
    };

    let f = File::create("./masterkey.cbor")?;
    serde_cbor::to_writer(f, &masterkey)?;

    let secrets: Secrets = [
        ("ABC".to_owned(), "DEF".to_owned()),
        ("XYZ".to_owned(), "ABC".to_owned()),
    ].iter().cloned().collect();

    let f = File::create("./secrets.cbor")?;
    serde_cbor::to_writer(f, &secrets)?;

    let secreta = "ABC".to_owned();
    let mut f = File::create("./keyreq_a.cbor")?;
    let buf = serde_cbor::to_vec(&secreta)?;
    f.write(&buf.len().to_be_bytes())?;
    f.write(&buf)?;

    let secretb = "XYZ".to_owned();
    let mut f = File::create("./keyreq_b.cbor")?;
    let buf = serde_cbor::to_vec(&secretb)?;
    f.write(&buf.len().to_be_bytes())?;
    f.write(&buf)?;

    let secs = vec!{"ABC", "XYZ"};
    let f = File::create("./secreq.yaml")?;
    serde_yaml::to_writer(f, &secs)?;

    Ok(())
}
