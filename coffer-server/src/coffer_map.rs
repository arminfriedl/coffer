//! Thread-safe coffer implementation backed by hash map

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

use std::collections::HashMap;

use coffer_common::coffer::*;

type ShardedCoffer = HashMap<String, HashMap<String, CofferValue>>;
pub struct CofferMap(RwLock<ShardedCoffer>);

impl CofferMap {
    pub fn new() -> CofferMap {
        CofferMap(RwLock::new(HashMap::new()))
    }

    fn read(&self) -> RwLockReadGuard<'_, ShardedCoffer> {
        self.0.read().unwrap()
    }

    fn write(&self) -> RwLockWriteGuard<'_, ShardedCoffer> {
        self.0.write().unwrap()
    }
}

impl Coffer for CofferMap {
    fn put(&mut self, key: CofferKey, value: CofferValue) -> CofferResult<()> {
        let mut lock = self.write();

        match lock.get_mut(&key.shard) {
            Some(shard) => {
                if shard.contains_key(&key.key) { Err(CofferError::Msg("Key exists")) }
                else { shard.insert(key.key, value); Ok(()) }
            }
            None => {
                lock.insert(key.shard.clone(), HashMap::new());
                lock.get_mut(&key.shard).unwrap().insert(key.key, value);
                Ok(())
            }
        }
    }

    fn push(&mut self, key: CofferKey, value: CofferValue) {
        let mut lock = self.write();

        match lock.get_mut(&key.shard) {
            Some(shard) => {
                shard.insert(key.key, value);
            }
            None => {
                lock.insert(key.shard.clone(), HashMap::new());
                lock.get_mut(&key.shard).unwrap().insert(key.key, value);
            }
        }
    }

    fn get(&self, key: &CofferKey) -> CofferResult<CofferValue> {
        let lock = self.read();

        let res = lock.get(&key.shard)
            .and_then( |shard| { shard.get(&key.key) } )
            .ok_or(CofferError::Msg("Key not found"))?;

        Ok(res.clone())
    }

    fn get_shard<T>(&self, shard: T) -> CofferResult<CofferShard>
    where T: AsRef<str>
    {
        let lock = self.read();

        let coffer_shard = lock.get(shard.as_ref())
            .ok_or(CofferError::Msg("Shard {} not found"))?;

        let mut res = CofferShard(Vec::new());

        for (k,v) in coffer_shard {
            res.0.push((k.clone(), v.clone()));
        }

        Ok(res)
    }
}

impl Default for CofferMap {
    fn default() -> Self {
        CofferMap::new()
    }
}
