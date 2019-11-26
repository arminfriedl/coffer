use std::collections::HashMap;
use sodiumoxide::crypto::box_;

use serde::{Serialize, Deserialize};

#[derive(Debug,Serialize,Deserialize)]
pub struct MasterKey (String, box_::SecretKey, box_::PublicKey);

#[derive(Debug,Serialize,Deserialize)]
pub struct ClientKey (String, box_::PublicKey);

#[derive(Default)]
struct KeyStore {
    keys: HashMap<String, ClientKey>
}

impl KeyStore {
    fn get(&self, key: &str) -> Option<&ClientKey> {
        self.keys.get(key)
    }
}

pub struct KeyRing {
    master: MasterKey,
    keystore: KeyStore
}

impl KeyRing {
    pub fn new(master: MasterKey) -> KeyRing {
        KeyRing {master, keystore: KeyStore::default()}
    }

    pub fn seal(&self, data: &[u8], nonce: &[u8], id: String) -> Vec<u8> {
        let nonce = box_::Nonce::from_slice(nonce).unwrap();
        let sender_sk = &self.master.1;
        let receiver_pk = &self.keystore.get(&id).unwrap().1;
        box_::seal(&data, &nonce, &receiver_pk, &sender_sk)
    }

    pub fn unseal(&self, data: &[u8], nonce: &[u8], id: String) -> Vec<u8> {
        let nonce = box_::Nonce::from_slice(nonce).unwrap();
        let receiver_sk = &self.master.1;
        let sender_pk = &self.keystore.get(&id).unwrap().1;
        box_::open(&data, &nonce, &sender_pk, &receiver_sk).unwrap()
    }

    pub fn add_key(&mut self, id: String, pubkey: [u8;32]) {
        self.keystore.keys.insert(id.clone(), ClientKey(id, box_::PublicKey::from_slice(&pubkey).unwrap()));
    }
}
