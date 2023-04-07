//! Module to wrap the openssl library for crypto functions

pub mod certificate;
pub mod hash;
pub mod signature;

use std::fmt::Display;

use crate::error::VerifierError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpensslErrorType {
    Keystore,
    Certificate,
    PublicKey,
    Time,
}

impl Display for OpensslErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Keystore => "Read Keystore",
            Self::Certificate => "Read Certificate",
            Self::PublicKey => "Public Key",
            Self::Time => "Time Error",
        };
        write!(f, "{s}")
    }
}

pub type OpensslError = VerifierError<OpensslErrorType>;
