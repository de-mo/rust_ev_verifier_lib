pub mod setup;

use num::BigUint;

use crate::crypto_primitives::num_bigint::Hexa;
use crate::error::VerifierError;
use serde::de::{Deserialize, Deserializer, Error};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializeErrorType {
    MalformedJSON,
    FieldError,
}

impl Display for DeserializeErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MalformedJSON => "MalformedJSON",
            Self::FieldError => "FieldError",
        };
        write!(f, "{s}")
    }
}

type DeserializeError = VerifierError<DeserializeErrorType>;

pub trait FromJson: Sized {
    fn from_json(s: &String) -> Result<Self, DeserializeError>;
}

fn deserialize_string_hex_to_bigunit<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    BigUint::from_hexa(&buf).map_err(|e| Error::custom(e.message()))
}
