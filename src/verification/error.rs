use crate::error::{create_verifier_error, VerifierError};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationErrorType {
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFailureType {
    Failure,
}

macro_rules! create_verification_error {
    ($m: expr) => {
        create_verifier_error!(VerificationErrorType::Error, $m)
    };
    ($m: expr, $e: expr) => {
        create_verifier_error!(VerificationErrorType::Error, $m, $e)
    };
}
pub(crate) use create_verification_error;

macro_rules! create_verification_failure {
    ($m: expr) => {
        create_verifier_error!(VerificationFailureType::Failure, $m)
    };
    ($m: expr, $e: expr) => {
        create_verifier_error!(VerificationFailureType::Failure, $m, $e)
    };
}
pub(crate) use create_verification_failure;

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
