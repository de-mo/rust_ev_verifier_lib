//! Module to wrap the openssl library for crypto functions

mod certificate;
mod hash;
mod signature;

pub use certificate::*;
pub use hash::*;
pub use signature::*;

use openssl::error::ErrorStack;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpensslError {
    #[error("IO error caused by {source}: {msg}")]
    IO { msg: String, source: io::Error },
    #[error("Keystore error {msg} caused by {source}")]
    Keystore { msg: String, source: ErrorStack },
    #[error("Keystore {0} has not list of CA")]
    KeyStoreMissingCAList(PathBuf),
    #[error("The ca with name {name} is not present in the Keystore {path}")]
    KeyStoreMissingCA { path: PathBuf, name: String },
    #[error("Error reading public key in the certificate {name} caused by {source}")]
    CertificateErrorPK { name: String, source: ErrorStack },
    #[error("Error of time during time check of the certificate {name} caused by {source}")]
    CertificateErrorTime { name: String, source: ErrorStack },
    #[error(transparent)]
    PublicKeyError(#[from] ErrorStack),
    #[error("{msg} caused by {source}")]
    SignatureVerify { msg: String, source: ErrorStack },
    //Certificate,
    //PublicKey,
    //Time,
    //VerifiySignature,
}
