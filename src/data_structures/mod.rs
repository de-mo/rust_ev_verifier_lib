//! Module to collect data structures of the verifier

pub mod error;
pub mod setup;
pub mod setup_or_tally;
pub mod tally;

use num_bigint::BigUint;

use self::{
    error::DeserializeError,
    setup::{
        control_component_code_shares_payload::ControlComponentCodeSharesPayload,
        control_component_public_keys_payload::ControlComponentPublicKeysPayload,
        election_event_context_payload::ElectionEventContextPayload,
        encryption_parameters_payload::EncryptionParametersPayload,
        setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
        setup_component_tally_data_payload::SetupComponentTallyDataPayload,
        setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
        VerifierSetupData, VerifierSetupDataType,
    },
    setup_or_tally::SetupOrTally,
    tally::{VerifierTallyData, VerifierTallyDataType},
};
use crate::crypto_primitives::byte_array::{ByteArray, Decode};
use crate::crypto_primitives::num_bigint::Hexa;
use serde::de::{Deserialize, Deserializer, Error};
use serde::Deserialize as Deserialize2;

/// The type VerifierData implement an option between [VerifierSetupData] and [VerifierTallyData]
pub type VerifierData = SetupOrTally<VerifierSetupData, VerifierTallyData>;

/// The type VerifierDataType implement an option between [VerifierSetupDataType] and [VerifierTallyDataType]
pub type VerifierDataType = SetupOrTally<VerifierSetupDataType, VerifierTallyDataType>;

macro_rules! create_verifier_data_type {
    ($p: ident, $s: ident) => {
        VerifierDataType::$p(VerifierSetupDataType::$s)
    };
}
pub(crate) use create_verifier_data_type;

/// Trait implementing the collection of the specific data type from the enum object
pub trait VerifierDataTrait {
    /// Get the EncryptionParametersPayload is the enum is from correct type. Else give None
    fn encryption_parameters_payload(&self) -> Option<&EncryptionParametersPayload>;
    fn setup_component_public_keys_payload(&self) -> Option<&SetupComponentPublicKeysPayload>;
    fn election_event_context_payload(&self) -> Option<&ElectionEventContextPayload>;
    fn setup_component_tally_data_payload(&self) -> Option<&SetupComponentTallyDataPayload>;
    fn control_component_public_keys_payload(&self) -> Option<&ControlComponentPublicKeysPayload>;
    fn setup_component_verification_data_payload(
        &self,
    ) -> Option<&SetupComponentVerificationDataPayload>;
    fn control_component_code_shares_payload(&self) -> Option<&ControlComponentCodeSharesPayload>;
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

    fn setup_component_tally_data_payload(&self) -> Option<&SetupComponentTallyDataPayload> {
        match self {
            VerifierData::Setup(d) => d.setup_component_tally_data_payload(),
            VerifierData::Tally(_) => None,
        }
    }

    fn control_component_public_keys_payload(&self) -> Option<&ControlComponentPublicKeysPayload> {
        match self {
            VerifierData::Setup(d) => d.control_component_public_keys_payload(),
            VerifierData::Tally(_) => None,
        }
    }

    fn setup_component_verification_data_payload(
        &self,
    ) -> Option<&SetupComponentVerificationDataPayload> {
        match self {
            VerifierData::Setup(d) => d.setup_component_verification_data_payload(),
            VerifierData::Tally(_) => None,
        }
    }

    fn control_component_code_shares_payload(&self) -> Option<&ControlComponentCodeSharesPayload> {
        match self {
            VerifierData::Setup(d) => d.control_component_code_shares_payload(),
            VerifierData::Tally(_) => None,
        }
    }
}

impl VerifierDataType {
    /// Read VerifierDataType from a String as JSON
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

    BigUint::from_hexa_string(&buf).map_err(|e| Error::custom(e.message()))
}

fn deserialize_seq_string_hex_to_seq_bigunit<'de, D>(
    deserializer: D,
) -> Result<Vec<BigUint>, D::Error>
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
                let r_b = BigUint::from_hexa_string(&v).map_err(|e| A::Error::custom(e))?;
                vec.push(r_b);
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
                let r_b = ByteArray::base64_decode(&v).map_err(|e| A::Error::custom(e))?;
                vec.push(r_b);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

fn deserialize_seq_seq_string_hex_to_seq_seq_bigunit<'de, D>(
    deserializer: D,
) -> Result<Vec<Vec<BigUint>>, D::Error>
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
                    let r_b = BigUint::from_hexa_string(&x).map_err(|e| A::Error::custom(e))?;
                    inner_vec.push(r_b);
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
    pub signature_contents: String,
}

pub trait SignatureTrait {
    fn get_signature_struct(&self) -> &Signature;
    fn get_signature(&self) -> ByteArray {
        ByteArray::base64_decode(&self.get_signature_struct().signature_contents).unwrap()
    }
}

#[derive(Deserialize2, Debug, Clone)]
pub struct ProofUnderline {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_e")]
    pub e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_z")]
    pub z: BigUint,
}

#[derive(Deserialize2, Debug, Clone)]
pub struct Proof {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub z: BigUint,
}

impl From<&ProofUnderline> for Proof {
    fn from(value: &ProofUnderline) -> Self {
        Proof {
            e: value.e.clone(),
            z: value.z.clone(),
        }
    }
}

#[derive(Deserialize2, Debug, Clone)]
pub struct ExponentiatedEncryptedElement {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub gamma: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub phis: Vec<BigUint>,
}
