//! Module to collect data structures of the verifier
//TODO Document the module

pub mod error;
pub mod setup;
pub mod setup_or_tally;
pub mod tally;

use num::BigUint;

use self::{
    error::DeserializeError,
    setup::{
        election_event_context_payload::ElectionEventContextPayload,
        encryption_parameters_payload::EncryptionParametersPayload,
        setup_component_public_keys_payload::SetupComponentPublicKeysPayload, VerifierSetupData,
        VerifierSetupDataType,
    },
    setup_or_tally::SetupOrTally,
    tally::{VerifierTallyData, VerifierTallyDataType},
};
use crate::crypto_primitives::byte_array::{ByteArray, Decode};
use crate::crypto_primitives::num_bigint::Hexa;
use serde::de::{Deserialize, Deserializer, Error};
use serde::Deserialize as Deserialize2;

/*
pub enum VerifierData {
    Setup(setup::VerifierSetupData),
    Tally,
} */

pub type VerifierData = SetupOrTally<VerifierSetupData, VerifierTallyData>;
pub type VerifierDataType = SetupOrTally<VerifierSetupDataType, VerifierTallyDataType>;

macro_rules! create_verifier_data_type {
    ($p: ident, $s: ident) => {
        VerifierDataType::$p(VerifierSetupDataType::$s)
    };
}
pub(crate) use create_verifier_data_type;

pub trait VerifierDataTrait {
    fn encryption_parameters_payload(&self) -> Option<&EncryptionParametersPayload>;
    fn setup_component_public_keys_payload(&self) -> Option<&SetupComponentPublicKeysPayload>;
    fn election_event_context_payload(&self) -> Option<&ElectionEventContextPayload>;
}

impl VerifierDataTrait for VerifierData {
    fn encryption_parameters_payload(&self) -> Option<&EncryptionParametersPayload> {
        match self {
            VerifierData::Setup(d) => d.encryption_parameters_payload(),
            VerifierData::Tally(_) => None,
        }
    }

    fn setup_component_public_keys_payload(&self) -> Option<&SetupComponentPublicKeysPayload> {
        match self {
            VerifierData::Setup(d) => d.setup_component_public_keys_payload(),
            VerifierData::Tally(_) => None,
        }
    }

    fn election_event_context_payload(&self) -> Option<&ElectionEventContextPayload> {
        match self {
            VerifierData::Setup(d) => d.election_event_context_payload(),
            VerifierData::Tally(_) => None,
        }
    }
}

impl VerifierDataType {
    pub fn verifier_data_from_json(&self, s: &String) -> Result<VerifierData, DeserializeError> {
        match self {
            VerifierDataType::Setup(t) => {
                t.verifier_data_from_json(s).map(|r| VerifierData::Setup(r))
            }
            VerifierDataType::Tally(_) => todo!(),
        }
    }
}

pub trait DataStructureTrait {
    fn from_json(s: &String) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    fn to_encryption_parameters_payload(&self) -> Option<&EncryptionParametersPayload> {
        None
    }
}

macro_rules! implement_trait_data_structure {
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
use implement_trait_data_structure;

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

#[derive(Deserialize2, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    signature_contents: String,
}

#[derive(Deserialize2, Debug, Clone)]
pub struct SchnorrProofUnderline {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_e")]
    e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_z")]
    z: BigUint,
}

#[derive(Deserialize2, Debug, Clone)]
pub struct SchnorrProof {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    z: BigUint,
}

#[derive(Deserialize2, Debug, Clone)]
pub struct ExponentiatedEncryptedElement {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    gamma: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    phis: Vec<BigUint>,
}
