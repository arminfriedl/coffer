//! A storage container for client data
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::fmt::Debug;
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

        coffer.from_toml_table(&clients);

        coffer
    }

    fn from_toml_table(&mut self, toml_table: &toml::value::Table) {
        // table has an no id, recourse into subtables
        if toml_table.get("id").is_none() {
            for (_key, val) in toml_table.iter() {
                match val {
                    TomlValue::Table(subtable) => {
                        self.from_toml_table(subtable);
                    },
                    _ => panic!{"Invalid secrets file"}
                }
            }

            return;
        }

        /*
         * Parse a single shard/table, this is known to have an id
         *
         * [files]
         * id = "ABC-DEF-GHE"
         * secret_string = "secret value1"
         * secret_int = 12345
         * secret_bool = true
         */
        let shard = toml_table.get("id").and_then(|id| id.as_str()).unwrap();

        for (key, val) in toml_table {
            if "id" == key { continue } // ids are for sharding

            let value = match val {
                TomlValue::String(s) => CofferValue::String(s.to_owned()),
                TomlValue::Integer(i) => CofferValue::Integer(*i as i32),
                TomlValue::Float(f) => CofferValue::Float(*f as f32),
                TomlValue::Boolean(b) => CofferValue::Boolean(*b),
                _ => panic!{"Value {:?} unsupported", val}
            };

            let key =  key.to_owned();
            let shard = shard.to_string();
            self.put(CofferKey{shard, key}, value).unwrap();
        }
    }
}
