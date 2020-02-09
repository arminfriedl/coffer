//! A storage container for client data
//!
//! Coffer supports separated client data by `CofferShard`s. Content of a shard
//! is a key-value store with typed values.
//!
//! # Coffer files
//! A `Coffer` can be read from a [toml](https://github.com/toml-lang/)
//! file in a specific format.
//!
//! ## Shards
//! A `CofferShard` is identified by a toml table with a field `id` containing
//! the unique shard id.
//!
//! Tables can be nested for grouping shards together. The grouping is not
//! necessarily reflected in the deserialized `Coffer`, as shards can be
//! uniquely identified by their id.
//!
//! Shards (tables with an id) cannot be nested.
//!
//! A simple shard with no data
//! ```toml
//!   [app]
//!   id = "1"
//! ```
//!
//! Grouped shard
//! ```toml
//!   [app]
//!   [app.frontend]
//!   id = "1"
//!
//!   [app.backend]
//!   id = "2"
//! ```
//!
//! Nested shards (invalid)
//! ```toml
//!   [app]
//!   id = "1" # app is a shard since it has an id
//!   [app.frontend] # invalid, can't nest shards inside other shards
//!   id = "2"
//! ```
//!
//! ## Values
//! Shards can contain a subset of toml values. The currently supported toml
//! values are:
//! - [String](https://github.com/toml-lang/toml#user-content-string)
//! - [Integer](https://github.com/toml-lang/toml#user-content-integer)
//! - [Float](https://github.com/toml-lang/toml#user-content-float)
//! - [Boolean](https://github.com/toml-lang/toml#user-content-boolean)
//!
//! ## Example
//! ```toml
//!   [app]
//!   [app.frontend]
//!   id = "1"
//!   password = "admin"
//!   font_size = 1.4
//!
//!   [app.backend]
//!   id = "2"
//!   cors = true
//!
//!   [database]
//!   id = "0"
//!   user = "root"
//!   passwort = "toor"
//! ```
//!
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::{
  fmt::Debug,
  fs::File,
  io::{BufReader, Read},
  path::Path,
};

use quick_error::quick_error;
use toml::Value as TomlValue;
use serde::{Serialize, Deserialize};

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

/// Values supported by `Coffer`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CofferValue {
    /// A UTF-8 encoded string
    String(String),
    /// A 32-bit integer
    Integer(i32),
    /// A 32-bit float
    Float(f32),
    /// A boolean value
    Boolean(bool)
}

/// A `CofferKey` defining the shard and the key into the kv-store
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CofferKey {
    pub shard: String,
    pub key: String
}

/// A key-value store for client data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CofferShard(pub Vec<(String, CofferValue)>);

/// Trait for `Coffer`
pub trait Coffer {
    /// Put `value` at `key`. Errors if there is already a value for `key`.
    fn put(&mut self, key: CofferKey, value: CofferValue) -> CofferResult<()>;

    /// Push `value` to `key`. Replaces existing values.
    fn push(&mut self, key: CofferKey, value: CofferValue);

    /// Retrieve `value` at path. `None` if there is no `value` for `key`.
    fn get(&self, key: &CofferKey) -> Option<CofferValue>;

    /// Retrieve an entire shard. `None` if there is no `CofferShard` for `shard`.
    fn get_shard<T>(&self, shard: T) -> Option<CofferShard>
    where T: AsRef<str>;

    /// Deserializes a `Coffer` from a toml file
    fn from_toml_path(toml_path: &Path) -> Self
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
