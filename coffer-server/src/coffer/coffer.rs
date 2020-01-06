#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use coffer_common::certificate::Certificate;

pub struct Coffer {
    certificate: Certificate,
}

impl Coffer {
    /// Create a new, empty `Coffer` with a generated certificate
    pub fn new() -> Coffer {
        Coffer {certificate: Certificate::new()}
    }

    /// Create a new `Coffer` with certificate
    pub fn new_with_certificate (certificate: Certificate) {
        Coffer {certificate.into()}
    }
}
