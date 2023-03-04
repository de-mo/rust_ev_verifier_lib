//! Module to collect data structures of the verifier
//TODO Document the module

pub mod setup;

use num::BigUint;

use crate::crypto_primitives::byte_array::{ByteArray, Decode};
use crate::crypto_primitives::num_bigint::Hexa;
use crate::error::VerifierError;
use serde::de::{Deserialize, Deserializer, Error};
use serde::Deserialize as Deserialize2;
use std::fmt::Display;

pub enum VerifierData {
    Setup(setup::VerifierSetupData),
    Tally,
}

pub trait VerifierDataTrait {
    fn new_empty(&self) -> Self;

    fn new_from_json(&self, s: &String) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    fn is_some(&self) -> bool;
    fn is_none(&self) -> bool {
        !self.is_some()
    }
}

impl VerifierDataTrait for VerifierData {
    fn new_empty(&self) -> Self {
        match self {
            VerifierData::Setup(t) => VerifierData::Setup(t.new_empty()),
            VerifierData::Tally => todo!(),
        }
    }

    fn new_from_json(&self, s: &String) -> Result<Self, DeserializeError> {
        match self {
            VerifierData::Setup(t) => t.new_from_json(s).map(|r| VerifierData::Setup(r)),
            VerifierData::Tally => todo!(),
        }
    }

    fn is_some(&self) -> bool {
        match self {
            VerifierData::Setup(r) => r.is_some(),
            VerifierData::Tally => todo!(),
        }
    }
}

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

type DeserializeError = VerifierError<DeserializeErrorType>;

pub trait DataStructureTrait {
    fn from_json(s: &String) -> Result<Self, DeserializeError>
    where
        Self: Sized;
}

macro_rules! implement_trait_fromjson {
    ($s: ty) => {
        impl DataStructureTrait for $s {
            fn from_json(s: &String) -> Result<Self, DeserializeError> {
                serde_json::from_str(s).map_err(|e| {
                    create_verifier_error!(
                        DeserializeErrorType::JSONError,
                        format!("Cannot deserialize json"),
                        e
                    )
                })
            }
        }
    };
}
use implement_trait_fromjson;

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

fn deserialize_seq_string_64_to_seq_bytearray<'de, D>(
    deserializer: D,
) -> Result<Vec<ByteArray>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<ByteArray>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element())? {
                let r_b = ByteArray::base64_decode(&v);
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

fn deserialize_seq_seq_string_hex_to_seq_seq_bigunit<'de, D>(
    deserializer: D,
) -> Result<Vec<Vec<num::BigUint>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Vec<BigUint>>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element::<Vec<String>>())? {
                let mut inner_vec = Vec::new();
                for x in v {
                    let r_b = BigUint::from_hexa(&x);
                    if r_b.is_err() {
                        return Err(A::Error::custom(r_b.unwrap_err()));
                    }
                    inner_vec.push(r_b.unwrap());
                }
                vec.push(inner_vec.to_owned());
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

#[derive(Deserialize2, Debug)]
pub struct SchnorrProofUnderline {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_e")]
    e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_z")]
    z: BigUint,
}

#[derive(Deserialize2, Debug)]
pub struct SchnorrProof {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    z: BigUint,
}

#[derive(Deserialize2, Debug)]
pub struct ExponentiatedEncryptedElement {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    gamma: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    phis: Vec<BigUint>,
}
