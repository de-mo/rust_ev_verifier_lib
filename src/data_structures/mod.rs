pub mod setup;

use num::BigUint;

use crate::crypto_primitives::num_bigint::Hexa;
use crate::error::VerifierError;
use serde::de::{Deserialize, Deserializer, Error};
use serde::Deserialize as Deserialize2;
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

fn deserialize_seq_string_hex_to_seq_bigunit<'de, D>(
    deserializer: D,
) -> Result<Vec<num::BigUint>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<BigUint>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element())? {
                let r_b = BigUint::from_hexa(&v);
                if r_b.is_err() {
                    return Err(A::Error::custom(r_b.unwrap_err()));
                }
                vec.push(r_b.unwrap());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

#[derive(Deserialize2, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    signature_contents: String,
}
