#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

pub type Secrets = HashMap<String, String>;

pub fn parse_from_path(path: &PathBuf, keep: bool) -> Result<Secrets, Box<dyn Error>> {
    let mut sec_file = File::open(path)?;

    let mut sec_data = Vec::new();
    sec_file.read_to_end(&mut sec_data)?;
    if !keep { std::fs::remove_file(path)?; };

    Ok(serde_cbor::from_slice::<Secrets>(&sec_data)?)
}
