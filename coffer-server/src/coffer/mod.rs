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
            from()}
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

