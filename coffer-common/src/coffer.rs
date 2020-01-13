//! A storage container for client data

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

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
    /// An opaque blob of data
    Blob(Vec<u8>)
}

/// A path to a value
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CofferPath(pub Vec<String>);

/// Interface for interacting with a `Coffer`
pub trait Coffer {
    /// Put `value` at `path`. Errors if there is already a value at `path`.
    fn put(&mut self, path: CofferPath, value: CofferValue) -> CofferResult<()>;

    /// Push `value` to `path`. Replaces existing values silently.
    fn push(&mut self, path: CofferPath, value: CofferValue);

    /// Retrieve `value` at path. Errors if there is no `value` at path.
    fn get(&self, path: CofferPath) -> CofferResult<CofferValue>;
}

impl <T> From<Vec<T>> for CofferPath
where T: AsRef<str>
{
    fn from(val: Vec<T>) -> Self {
        CofferPath::from(&val)
    }
}

impl <T> From<&Vec<T>> for CofferPath
where T: AsRef<str>
{
    fn from(val: &Vec<T>) -> Self {
        let col = val.iter().map(|p| {(*p).as_ref().to_owned()}).collect();
        CofferPath(col)
    }
}
