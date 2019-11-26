use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sealedbox;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct MasterKey {
    public_key: box_::PublicKey,
    secret_key: box_::SecretKey,
}

struct ClientKey {
    id: String,
    public_key: box_::PublicKey,
}

pub type Secrets = HashMap<String, String>;
pub type Secs = Vec<String>;

fn main() -> Result<(), Box<dyn Error>> {
    let keypair = box_::gen_keypair();
    let masterkey: MasterKey = MasterKey {
        public_key: keypair.0,
        secret_key: keypair.1,
    };

    let f = File::create("./masterkey.cbor")?;
    serde_cbor::to_writer(f, &masterkey)?;

    let secrets: Secrets = [
        ("ABC".to_owned(), "DEF".to_owned()),
        ("XYZ".to_owned(), "ABC".to_owned()),
    ]
    .iter()
    .cloned()
    .collect();

    let sc_res = serde_cbor::to_vec(&secrets)?;
    let sc_res = sealedbox::seal(&sc_res, &masterkey.public_key);
    let mut f = File::create("./secrets.cbor")?;
    f.write(&sc_res)?;
    f.flush()?;

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

    let secs = vec!["ABC", "XYZ"];
    let f = File::create("./secreq.yaml")?;
    serde_yaml::to_writer(f, &secs)?;

    Ok(())
}
