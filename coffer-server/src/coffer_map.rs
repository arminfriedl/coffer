//! Thread-safe coffer implementation backed by hash map

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

use std::collections::hash_map::{HashMap, Entry};

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
                match shard.entry(key.key) {
                    Entry::Occupied(_) => Err(CofferError::Msg("Key exists")),
                    Entry::Vacant(v) => { v.insert(value); Ok(()) }
                }
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

    fn get(&self, key: &CofferKey) -> Option<CofferValue> {
        let lock = self.read();

        lock.get(&key.shard)
            .and_then( |shard| { shard.get(&key.key) } )
            .map(|o| o.clone())
    }

    fn get_shard<T>(&self, shard: T) -> Option<CofferShard>
    where T: AsRef<str>
    {
        let lock = self.read();

        debug!{"Coffer {:?}", *lock}

        let map_to_vec = |map: &HashMap<String, CofferValue>| {
            map.iter()
               .map(|(k,v)| (k.clone(), v.clone()))
               .collect::<Vec<(String, CofferValue)>>()
        };

        lock.get(shard.as_ref())
            .and_then(|s| Some(CofferShard(map_to_vec(s))))
    }
}

impl Default for CofferMap {
    fn default() -> Self {
        CofferMap::new()
    }
}
