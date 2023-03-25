use std::fmt::Display;

use crate::error::VerifierError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializeErrorType {
    JSONError,
}

impl Display for DeserializeErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::JSONError => "JSONError",
        };
        write!(f, "{s}")
    }
}

pub type DeserializeError = VerifierError<DeserializeErrorType>;
