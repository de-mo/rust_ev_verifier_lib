use crate::error::VerifierError;
use std::fmt::Display;

#[derive(Debug)]
pub enum VerificationErrorType {
    Error,
}

#[derive(Debug)]
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
