//! Common certificate handling and encryption

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::path::Path;
use std::io::BufReader;
use std::fs::File;
use std::fmt::{Debug, Formatter};

use quick_error::quick_error;

use seckey::SecKey;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sealedbox;
use serde::{Serialize, Deserialize};
use serde_cbor;

quick_error! {
    #[derive(Debug)]
    pub enum CertificateError {
        Cbor(err: serde_cbor::Error) {
            from()
        }
        Io(err: std::io::Error) {
            from()
        }
        SecKey
        Crypto
    }
}

/// A secure container for certificates
///
/// # Certificate
///
/// A certificate consists of a public and a private key in a secure memory
/// area. With a certificate data sealed and opened.
pub struct Certificate {
    inner: SecKey<CertificateInner>
}

#[derive(Serialize, Deserialize)]
struct CertificateInner {
    public_key: box_::PublicKey,
    private_key: box_::SecretKey
}

impl Debug for CertificateInner {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "<Certificate Hidden>")
    }
}

impl Certificate {
    pub fn new() -> Result<Certificate, CertificateError> {
        debug!{"Generating new certificate"}
        let (public_key, private_key) = box_::gen_keypair();

        let inner_cert = CertificateInner{public_key, private_key};
        let inner = SecKey::new(inner_cert).map_err(|_| CertificateError::SecKey)?;

        Ok(Certificate{inner})
    }

    pub fn new_from_cbor<T: AsRef<Path>>(path: T) -> Result<Certificate, CertificateError> {
        debug!{"Reading certificate from {}", path.as_ref().display()}
        let f = File::open(path)?;

        let inner_cert = serde_cbor::from_reader(BufReader::new(f))?;
        let inner = SecKey::new(inner_cert).map_err(|_| CertificateError::SecKey)?;

        Ok(Certificate{inner})
    }

    pub fn open(&self, c: &[u8]) -> Result<Vec<u8>, CertificateError> {
        let pk = &self.inner.read().public_key;
        let sk = &self.inner.read().private_key;

        debug!{"Opening sealed box"};
        sealedbox::open(c, pk, sk)
            .map_err(|_| CertificateError::Crypto)
    }

    fn seal(&self, m: &[u8]) -> Vec<u8> {
        let pk = &self.inner.read().public_key;

        debug!{"Sealing box"}
        sealedbox::seal(m, pk)
    }
}

impl <T: AsRef<Path>> From<T> for Certificate {
    fn from(path: T) -> Self {
        Certificate::new_from_cbor(&path)
            .expect(&format!{"Could not read certificate from {}", path.as_ref().display()})
    }
}
