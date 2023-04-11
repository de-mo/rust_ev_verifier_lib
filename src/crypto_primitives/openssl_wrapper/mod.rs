//! Module to wrap the openssl library for crypto functions

pub mod certificate;
pub mod hash;
pub mod signature;

use crate::error::VerifierError;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpensslErrorType {
    Keystore,
    Certificate,
    PublicKey,
    Time,
    VerifiySignature,
}

impl Display for OpensslErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Keystore => "Read Keystore",
            Self::Certificate => "Read Certificate",
            Self::PublicKey => "Public Key",
            Self::Time => "Time Error",
            Self::VerifiySignature => "Verify Signature",
        };
        write!(f, "{s}")
    }
}

pub type OpensslError = VerifierError<OpensslErrorType>;
