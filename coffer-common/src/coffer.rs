//! A storage container for client data
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};

use toml::Value as TomlValue;

use serde::{Serialize, Deserialize};

use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum CofferError {
        Msg(err: &'static str) {
            from(err)
            display("{}", err)
        }
        Other(err: Box<dyn std::error::Error>) {
            cause(&**err)
        }
    }
}

pub type CofferResult<T> = Result<T, CofferError>;

/// Values supported by a `Coffer`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CofferValue {
    /// A UTF-8 encoded string
    String(String),
    /// A 32-bit integer
    Integer(i32),
    /// A 32-bit float
    Float(f32),
    // A boolean
    Boolean(bool)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CofferKey {
    pub shard: String,
    pub key: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CofferShard(pub Vec<(String, CofferValue)>);

/// Interface for interacting with a `Coffer`
pub trait Coffer {
    /// Put `value` at `path`. Errors if there is already a value at `path`.
    fn put(&mut self, key: CofferKey, value: CofferValue) -> CofferResult<()>;

    /// Push `value` to `path`. Replaces existing values.
    fn push(&mut self, key: CofferKey, value: CofferValue);

    /// Retrieve `value` at path. Errors if there is no `value` at path.
    fn get(&self, key: &CofferKey) -> CofferResult<CofferValue>;

    /// Retrieve `value` at path. Errors if there is no `value` at path.
    fn get_shard<T>(&self, shard: T) -> CofferResult<CofferShard>
    where T: AsRef<str>;

    fn from_toml_file(toml_path: &Path) -> Self
    where Self: Coffer + Default
    {
        // read the secrets file into a temporary string
        let mut file = BufReader::new(File::open(toml_path).unwrap());
        let mut secrets_buf = String::new();
        file.read_to_string(&mut secrets_buf).unwrap();

        Coffer::from_toml(&secrets_buf)
    }

    fn from_toml(toml: &str) -> Self
    where Self: Coffer + Default
    {
        // call implementation to create an empty coffer
        let mut coffer = Self::default();

        // parse the string into a toml Table
        let clients: toml::value::Table = match toml.parse::<TomlValue>().unwrap() {
            TomlValue::Table(t) => t,
            _ => panic!{"Invalid secrets file"}
        };

        /*
         * Walk through the table of clients, where each client is a table which
         * is either empty, or contains a table with at least an id and any
         * number of secrets
         *
         * # Example:
         *
         * files.id = "AAAA-BBBB-CCCC"
         * pad.id = "FFFF-EEEE-DDDD"
         *
         * [files]
         * secret_string = "secret value1"
         * secret_int = 12345
         * secret_bool = true
         */
        for (_k, v) in clients {

            let client = match v {
                TomlValue::Table(t) => t,
                _ => panic!{"Invalid secrets file"}
            };

            for (k, v) in client.iter() {

                if "id" == k { continue } // ids are for sharding

                let value = match v {
                    TomlValue::String(s) => CofferValue::String(s.to_owned()),
                    TomlValue::Integer(i) => CofferValue::Integer(*i as i32),
                    TomlValue::Float(f) => CofferValue::Float(*f as f32),
                    TomlValue::Boolean(b) => CofferValue::Boolean(*b),
                    _ => panic!{"Value {:?} unsupported", v}
                };

                match client.get("id") {
                    Some(TomlValue::String(shard)) => {
                        let shard = shard.to_owned();
                        let key =  k.to_owned();
                        coffer.put(CofferKey{shard, key}, value).unwrap();
                    },
                    _ => panic!{"Invalid secrets file"}
                }
            }
        }

        return coffer;
    }
}
