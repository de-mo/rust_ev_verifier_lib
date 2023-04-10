//! Module implementing the errors of the verifications
//!
use crate::error::VerifierError;
use std::fmt::Display;

/// Macro to create a verification error (with or without embedded error)
macro_rules! create_verification_error {
    ($m: expr) => {
        create_verifier_error!(VerificationErrorType::Error, $m)
    };
    ($m: expr, $e: expr) => {
        create_verifier_error!(VerificationErrorType::Error, $m, $e)
    };
}
pub(crate) use create_verification_error;

/// Macro to create a verification failure (with or without embedded error)
macro_rules! create_verification_failure {
    ($m: expr) => {
        create_verifier_error!(VerificationFailureType::Failure, $m)
    };
    ($m: expr, $e: expr) => {
        create_verifier_error!(VerificationFailureType::Failure, $m, $e)
    };
}
pub(crate) use create_verification_failure;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationErrorType {
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFailureType {
    Failure,
}

impl Display for VerificationErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Error => "Error on test",
        };
        write!(f, "{s}")
    }
}

impl Display for VerificationFailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Failure => "Failure on test",
        };
        write!(f, "{s}")
    }
}

pub type VerificationError = VerifierError<VerificationErrorType>;
pub type VerificationFailure = VerifierError<VerificationFailureType>;
