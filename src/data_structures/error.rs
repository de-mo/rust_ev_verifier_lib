//! Module implementing the errors for the data structures

use crate::error::VerifierError;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializeErrorType {
    FileError,
    JSONError,
    XMLError,
}

impl Display for DeserializeErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::FileError => "File Error",
            Self::JSONError => "JSON error",
            Self::XMLError => "XML error",
        };
        write!(f, "{s}")
    }
}

pub type DeserializeError = VerifierError<DeserializeErrorType>;
