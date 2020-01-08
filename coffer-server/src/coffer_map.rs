//! A simple, thread-safe coffer implementation backed by a hash map

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::sync::RwLock;
use std::collections::HashMap;

use coffer_common::coffer::*;

pub struct CofferMap {
    coffer: RwLock<HashMap<CofferPath, CofferValue>>
}

impl CofferMap {
    pub fn new() -> CofferMap {
        CofferMap {
            coffer: RwLock::new(HashMap::new())
        }
    }
}

impl Coffer for CofferMap {
    fn put(&mut self, path: CofferPath, value: CofferValue) -> CofferResult<()> {
        let mut lock = self.coffer.write().unwrap();

        match (*lock).contains_key(&path) {
            true => Err(CofferError::Msg("test")),
            false => {(*lock).insert(path, value); Ok(())}
        }
    }

    fn push(&mut self, path: CofferPath, value: CofferValue) {
        let mut lock = self.coffer.write().unwrap();

        (*lock).insert(path, value);
    }

    fn get(&self, path: CofferPath) -> CofferResult<CofferValue> {
        let lock = self.coffer.read().unwrap();

        (*lock).get(&path)
            .and_then(|v| Some(v.clone()))
            .ok_or(CofferError::Msg("Key not found"))
    }
}

impl Default for CofferMap {
    fn default() -> Self {
        CofferMap::new()
    }
}
