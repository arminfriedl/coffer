//! A keypair contianer providing functionality for signing, encryption and
//! decryption
//!
//! # Base libraries
//! The cryptographic operations exposed by this module are based on the
//! [NaCl](http://nacl.cr.yp.to/) fork [libsodium](https://libsodium.org) as
//! exposed by the rust bindings [sodiumoxide](https://crates.io/crates/sodiumoxide).

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::{
    path::Path,
    io::BufReader,
    fs::File,
    ops::Deref
};

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
        SecKey {
            from(CertificateInner)
        }
        Crypto
    }
}

/// A secure container for a keypair
///
/// Secure means a best effort approach to:
/// - Prevent swapping memory to disk
/// - Zeroing out memory upon dropping
/// - Prevent other processes and buffer overflows to access the secure memory
///   area
///
/// These guarantees are currently *not* reliable. If you threat model contains
/// targeted attacks against coffer memory, additional precautions have to be
/// taken.
pub struct Certificate {
    inner: SecKey<CertificateInner>
}

// The SecKeyReadGuard prevents convenience methods for handing out references
// to private/public keys (reference outlives SecKeyReadGuard). Hence below
// macros are shortcut projections that can be used after a read guard is
// created

// Get the public key
macro_rules! pk {
  ($cert:ident) => {
    &$cert.inner.read().public_key
  };
}

// Get the private key
macro_rules! sk {
  ($cert:ident) => {
    &$cert.inner.read().private_key
  };
}

// Certificate and its inner SecKey own their
// raw pointer without any thread local behaviour
unsafe impl Send for Certificate {}
// After initialization, certificate is read-only
unsafe impl Sync for Certificate {}

#[derive(Serialize, Deserialize)]
struct CertificateInner {
    public_key: box_::PublicKey,
    private_key: box_::SecretKey
}

impl Certificate {
  /// Initialize with a generated keypair
    pub fn new() -> Result<Certificate, CertificateError> {
        debug!{"Generating new certificate"}
        let (public_key, private_key) = box_::gen_keypair();

        let inner_cert = CertificateInner{public_key, private_key};
        let inner = SecKey::new(inner_cert).map_err(|_| CertificateError::SecKey)?;

        Ok(Certificate{inner})
    }

    /// Initialize from a serialized certificate in [cbor](https://cbor.io/) format
    pub fn new_from_cbor<T: AsRef<Path>>(path: T) -> Result<Certificate, CertificateError> {
        debug!{"Reading certificate from {}", path.as_ref().display()}
        let f = File::open(path)?;

        let inner_cert = serde_cbor::from_reader(BufReader::new(f))?;
        let inner = SecKey::new(inner_cert).map_err(|_| CertificateError::SecKey)?;

        Ok(Certificate{inner})
    }

    /// Serialize a certificate to a file in [cbor](https://cbor.io/) format
    #[cfg(feature = "export")]
    pub fn to_cbor(&self) -> Result<Vec<u8>, CertificateError> {
        let read_guard = self.inner.read();

        let cbor = serde_cbor::to_vec(read_guard.deref())?;

        Ok(cbor)
    }

    /// Clone the bytes of the public key
    pub fn public_key(&self) -> Vec<u8> {
        pk!(self).as_ref().to_owned()
    }

    /// Clone the bytes of the private key
    #[cfg(feature = "export")]
    pub fn secret_key(&self) -> Vec<u8> {
        sk!(self).as_ref().to_owned()
    }

    /// Open a [sealed box](https://download.libsodium.org/doc/public-key_cryptography/sealed_boxes)
    pub fn open(&self, c: &[u8]) -> Result<Vec<u8>, CertificateError> {
        sealedbox::open(c, pk!{self}, sk!{self})
            .map_err(|_| CertificateError::Crypto)
    }

    /// Seal a message in a [sealed box](https://download.libsodium.org/doc/public-key_cryptography/sealed_boxes)
    pub fn seal(&self, message: &[u8]) -> Result<Vec<u8>, CertificateError> {
        Ok(sealedbox::seal(message, pk!{self}))
    }
}
