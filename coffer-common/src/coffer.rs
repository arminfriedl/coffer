//! A storage container for client data

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum CofferError {
        Coffer
    }
}

pub type CofferResult<T> = Result<T, CofferError>;

/// Values supported by a `Coffer`
pub enum CofferValue {
    /// A UTF-8 encoded string
    String(String),
    /// A 32-bit integer
    Integer(i32),
    /// An opaque blob of data
    Blob(Vec<u8>)
}

/// A path to a value
pub struct CofferPath(Vec<String>);

/// Interface for interacting with a `Coffer`
pub trait Coffer {
    /// Put `value` at `path`. Errors if there is already a value at `path`.
    fn put(path: CofferPath, value: CofferValue) -> CofferResult<()>;
    /// Push `value` to `path`. Replaces existing values silently.
    fn push(path: CofferPath, value: CofferValue);
    /// Retrieve `value` at path. Errors if there is no `value` at path.
    fn get(path: CofferPath) -> CofferResult<CofferValue>;
}

impl <T> From<&[T]> for CofferPath
where T: AsRef<str>
{
    fn from(val: &[T]) -> Self {
        let col = val.iter().map(|p| {(*p).as_ref().to_owned()}).collect();
        CofferPath(col)
    }
}
