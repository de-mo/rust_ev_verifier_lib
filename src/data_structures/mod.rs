//! Module to define the structure of the data and to read the data from the files into these structures
//!
//! The module is separate in two module: [setup] and [tally]
//!
pub mod common_types;
pub mod context;
pub mod dataset;
pub mod setup;
pub mod tally;
mod xml;

pub use self::{
    context::{
        control_component_public_keys_payload::ControlComponentPublicKeysPayload,
        election_event_configuration::ElectionEventConfiguration,
        election_event_context_payload::ElectionEventContextPayload,
        setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
        setup_component_tally_data_payload::SetupComponentTallyDataPayload, VerifierContextData,
        VerifierContextDataType,
    },
    dataset::DatasetType,
    setup::{
        control_component_code_shares_payload::ControlComponentCodeSharesPayload,
        setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
        VerifierSetupData, VerifierSetupDataType,
    },
    tally::{
        control_component_ballot_box_payload::ControlComponentBallotBoxPayload,
        control_component_shuffle_payload::ControlComponentShufflePayload,
        e_voting_decrypt::EVotingDecrypt, ech_0110::ECH0110, ech_0222::ECH0222,
        tally_component_shuffle_payload::TallyComponentShufflePayload,
        tally_component_votes_payload::TallyComponentVotesPayload, VerifierTallyData,
        VerifierTallyDataType,
    },
};
use crate::config::VerifierConfig;
use chrono::NaiveDateTime;
use common_types::CiphertextDef;
use quick_xml::{DeError as QuickXmDeError, Error as QuickXmlError};
use roxmltree::{Document, Error as RoXmlTreeError};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{elgamal::Ciphertext, Integer};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ByteArray, DecodeTrait, Hexa};
use serde::{
    de::{Deserialize as DeDeserialize, Deserializer, Error as SerdeError},
    Deserialize,
};
use std::path::Path;
use thiserror::Error;
pub use xml::XMLError;

// Enum representing the datza structure errors
#[derive(Error, Debug)]
pub enum DataStructureError {
    #[error("IO error {msg} -> caused by: {source}")]
    IO { msg: String, source: std::io::Error },
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    ParseRoXML { msg: String, source: RoXmlTreeError },
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    ParseQuickXML { msg: String, source: QuickXmlError },
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    ParseQuickXMLDE { msg: String, source: QuickXmDeError },
    #[error("Error parsing json {msg} -> caused by: {source}")]
    ParseJSON {
        msg: String,
        source: serde_json::Error,
    },
    #[error("Data error {0}")]
    DataError(String),
}

/// The type VerifierData implement an option between
/// [VerifierContextData], [VerifierSetupData] and [VerifierTallyData]
pub type VerifierData = DatasetType<VerifierContextData, VerifierSetupData, VerifierTallyData>;

/// The type VerifierDataType implement an option between
/// [VerifierContextDataType], [VerifierSetupDataType] and [VerifierTallyDataType]
pub type VerifierDataType =
    DatasetType<VerifierContextDataType, VerifierSetupDataType, VerifierTallyDataType>;

macro_rules! create_verifier_context_data_type {
    ($p: ident, $s: ident) => {
        VerifierDataType::$p(VerifierContextDataType::$s)
    };
}
pub(crate) use create_verifier_context_data_type;

macro_rules! create_verifier_setup_data_type {
    ($p: ident, $s: ident) => {
        VerifierDataType::$p(VerifierSetupDataType::$s)
    };
}
pub(crate) use create_verifier_setup_data_type;

macro_rules! create_verifier_tally_data_type {
    ($p: ident, $s: ident) => {
        VerifierDataType::$p(VerifierTallyDataType::$s)
    };
}
pub(crate) use create_verifier_tally_data_type;

/// Trait implementing the collection of the specific context data type from the enum object
pub trait VerifierContextDataTrait: Sized {
    fn setup_component_public_keys_payload(self) -> Option<SetupComponentPublicKeysPayload> {
        None
    }
    fn election_event_context_payload(self) -> Option<ElectionEventContextPayload> {
        None
    }
    fn setup_component_tally_data_payload(self) -> Option<SetupComponentTallyDataPayload> {
        None
    }
    fn control_component_public_keys_payload(self) -> Option<ControlComponentPublicKeysPayload> {
        None
    }
    fn election_event_configuration(self) -> Option<ElectionEventConfiguration> {
        None
    }
}

/// Trait implementing the collection of the specific setup data type from the enum object
pub trait VerifierSetupDataTrait: Sized {
    fn setup_component_verification_data_payload(
        self,
    ) -> Option<SetupComponentVerificationDataPayload> {
        None
    }
    fn control_component_code_shares_payload(self) -> Option<ControlComponentCodeSharesPayload> {
        None
    }
}

/// Trait implementing the collection of the specific tally data type from the enum object
pub trait VerifierTallyDataTrait: Sized {
    fn e_voting_decrypt(self) -> Option<EVotingDecrypt> {
        None
    }
    fn ech_0110(self) -> Option<ECH0110> {
        None
    }
    fn ech_0222(self) -> Option<ECH0222> {
        None
    }
    fn tally_component_votes_payload(self) -> Option<TallyComponentVotesPayload> {
        None
    }
    fn tally_component_shuffle_payload(self) -> Option<TallyComponentShufflePayload> {
        None
    }
    fn control_component_ballot_box_payload(self) -> Option<ControlComponentBallotBoxPayload> {
        None
    }

    fn control_component_shuffle_payload(self) -> Option<ControlComponentShufflePayload> {
        None
    }
}

/// A trait defining the necessary function to decode to the Verifier Data
pub trait VerifierDataDecode: Sized {
    /// Decode the data from a json string
    ///
    /// # Return
    /// The decoded data or [DataStructureError] if something wrong, e.g. if it is not allowed, or if an error
    /// occured during the decoding
    fn decode_json(_: &str) -> Result<Self, DataStructureError> {
        Err(DataStructureError::NotImplemented(
            "decode_json".to_string(),
        ))
    }

    /// Decode the data from a xml string
    ///
    /// # Return
    /// The decoded data or [DataStructureError] if something wrong, e.g. if it is not allowed, or if an error
    /// occured during the decoding
    fn decode_xml<'a>(_: &'a Document<'a>) -> Result<Self, DataStructureError> {
        Err(DataStructureError::NotImplemented("decode_xml".to_string()))
    }

    /// Prepare the streamin of data from a json file
    ///
    /// # Return
    /// The decoded data or [DataStructureError] if something wrong, e.g. if it is not allowed, or if an error
    /// occured during the decoding
    fn stream_json(_: &Path) -> Result<Self, DataStructureError> {
        Err(DataStructureError::NotImplemented(
            "stream_json".to_string(),
        ))
    }

    /// Prepare the streamin of data from a xml file
    ///
    /// # Return
    /// The decoded data or [DataStructureError] if something wrong, e.g. if it is not allowed, or if an error
    /// occured during the decoding
    fn stream_xml(_: &Path) -> Result<Self, DataStructureError> {
        Err(DataStructureError::NotImplemented("stream_xml".to_string()))
    }
}

/// Macro to automatically implement the DataStructureTrait for a type
macro_rules! implement_trait_verifier_data_json_decode {
    ($s: ty) => {
        impl VerifierDataDecode for $s {
            fn decode_json(s: &str) -> Result<Self, DataStructureError> {
                serde_json::from_str(s).map_err(|e| DataStructureError::ParseJSON {
                    msg: format!("Cannot deserialize json"),
                    source: e,
                })
            }
        }
    };
}
use implement_trait_verifier_data_json_decode;

impl VerifierContextDataTrait for VerifierData {
    fn setup_component_public_keys_payload(self) -> Option<SetupComponentPublicKeysPayload> {
        match self {
            VerifierData::Context(d) => d.setup_component_public_keys_payload(),
            _ => None,
        }
    }

    fn election_event_context_payload(self) -> Option<ElectionEventContextPayload> {
        match self {
            VerifierData::Context(d) => d.election_event_context_payload(),
            _ => None,
        }
    }

    fn setup_component_tally_data_payload(self) -> Option<SetupComponentTallyDataPayload> {
        match self {
            VerifierData::Context(d) => d.setup_component_tally_data_payload(),
            _ => None,
        }
    }

    fn control_component_public_keys_payload(self) -> Option<ControlComponentPublicKeysPayload> {
        match self {
            VerifierData::Context(d) => d.control_component_public_keys_payload(),
            _ => None,
        }
    }

    fn election_event_configuration(self) -> Option<ElectionEventConfiguration> {
        match self {
            VerifierData::Context(d) => d.election_event_configuration(),
            _ => None,
        }
    }
}

impl VerifierSetupDataTrait for VerifierData {
    fn setup_component_verification_data_payload(
        self,
    ) -> Option<SetupComponentVerificationDataPayload> {
        match self {
            VerifierData::Setup(d) => d.setup_component_verification_data_payload(),
            _ => None,
        }
    }

    fn control_component_code_shares_payload(self) -> Option<ControlComponentCodeSharesPayload> {
        match self {
            VerifierData::Setup(d) => d.control_component_code_shares_payload(),
            _ => None,
        }
    }
}

impl VerifierTallyDataTrait for VerifierData {
    fn e_voting_decrypt(self) -> Option<EVotingDecrypt> {
        match self {
            VerifierData::Tally(d) => d.e_voting_decrypt(),
            _ => None,
        }
    }
    fn ech_0110(self) -> Option<ECH0110> {
        match self {
            VerifierData::Tally(d) => d.ech_0110(),
            _ => None,
        }
    }
    fn ech_0222(self) -> Option<ECH0222> {
        match self {
            VerifierData::Tally(d) => d.ech_0222(),
            _ => None,
        }
    }
    fn tally_component_votes_payload(self) -> Option<TallyComponentVotesPayload> {
        match self {
            VerifierData::Tally(d) => d.tally_component_votes_payload(),
            _ => None,
        }
    }
    fn tally_component_shuffle_payload(self) -> Option<TallyComponentShufflePayload> {
        match self {
            VerifierData::Tally(d) => d.tally_component_shuffle_payload(),
            _ => None,
        }
    }
    fn control_component_ballot_box_payload(self) -> Option<ControlComponentBallotBoxPayload> {
        match self {
            VerifierData::Tally(d) => d.control_component_ballot_box_payload(),
            _ => None,
        }
    }
    fn control_component_shuffle_payload(self) -> Option<ControlComponentShufflePayload> {
        match self {
            VerifierData::Tally(d) => d.control_component_shuffle_payload(),
            _ => None,
        }
    }
}

#[allow(dead_code)]
fn deserialize_string_hex_to_integer<'de, D>(deserializer: D) -> Result<Integer, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    Integer::from_hexa_string(&buf).map_err(|e| SerdeError::custom(e.to_string()))
}

fn deserialize_string_base64_to_integer<'de, D>(deserializer: D) -> Result<Integer, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    ByteArray::base64_decode(&buf)
        .map_err(|e| SerdeError::custom(e.to_string()))
        .map(|e| e.into_integer())
}

fn deserialize_option_string_base64_to_option_integer<'de, D>(
    deserializer: D,
) -> Result<Option<Integer>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = Option::<String>::deserialize(deserializer)?;

    match buf {
        Some(buf) => ByteArray::base64_decode(&buf)
            .map_err(|e| SerdeError::custom(e.to_string()))
            .map(|e| Some(e.into_integer())),
        None => Ok(None),
    }
}

fn deserialize_string_string_to_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    NaiveDateTime::parse_from_str(&buf, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| SerdeError::custom(e.to_string()))
}

#[allow(dead_code)]
fn deserialize_seq_string_hex_to_seq_integer<'de, D>(
    deserializer: D,
) -> Result<Vec<Integer>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Integer>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element())? {
                let r_b = Integer::from_hexa_string(v).map_err(A::Error::custom)?;
                vec.push(r_b);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

#[allow(dead_code)]
fn deserialize_seq_string_base64_to_seq_bytearray<'de, D>(
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
                let r_b = ByteArray::base64_decode(v).map_err(A::Error::custom)?;
                vec.push(r_b);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

fn deserialize_seq_string_base64_to_seq_integer<'de, D>(
    deserializer: D,
) -> Result<Vec<Integer>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Integer>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element())? {
                let r_b = ByteArray::base64_decode(v).map_err(A::Error::custom)?;
                vec.push(r_b.into_integer());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

fn deserialize_seq_ciphertext<'de, D>(deserializer: D) -> Result<Vec<Ciphertext>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(with = "CiphertextDef")] Ciphertext);

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| a).collect())
}

#[allow(dead_code)]
fn deserialize_seq_seq_string_hex_to_seq_seq_integer<'de, D>(
    deserializer: D,
) -> Result<Vec<Vec<Integer>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Vec<Integer>>;

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
                    let r_b = Integer::from_hexa_string(&x).map_err(A::Error::custom)?;
                    inner_vec.push(r_b);
                }
                vec.push(inner_vec.to_owned());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

fn deserialize_seq_seq_string_base64_to_seq_seq_integer<'de, D>(
    deserializer: D,
) -> Result<Vec<Vec<Integer>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Vec<Integer>>;

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
                    let r_b = ByteArray::base64_decode(&x).map_err(A::Error::custom)?;
                    inner_vec.push(r_b.into_integer());
                }
                vec.push(inner_vec.to_owned());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

/// Verification of the length of unique ID according the expected length l_id
///
/// `name` is used for the error message
fn verifiy_domain_length_unique_id(uuid: &str, name: &str) -> Vec<String> {
    if uuid.len() != VerifierConfig::l_id() {
        return vec![format!(
            "The  length of {} {} must be {}",
            uuid,
            name,
            VerifierConfig::l_id()
        )];
    };
    vec![]
}

#[cfg(test)]
pub(super) mod test {
    /// Macro testing the data structure (read data, signature and verify domain)
    ///
    /// # Parameters
    /// $t: Name of the struct containing the data
    /// $f: Filename as str
    /// $fn_path: Function to get the path of the test data
    /// $ignored (optional): If the signature test is not working, can be ignored with the comment $ignored
    ///
    /// # usage:
    /// All the four macros have to be imported
    macro_rules! test_data_structure {
        ($t:ident, $f: literal, $fn_path: ident, $ignored: literal) => {
            fn get_data_res() -> Result<$t, DataStructureError> {
                let json = fs::read_to_string($fn_path().join($f)).unwrap();
                $t::decode_json(&json)
            }
            test_data_structure_read_data_set!();
            test_data_structure_verify_signature!($ignored);
            test_data_structure_verify_domain!();
        };
        ($t:ident, $f: literal, $fn_path: ident) => {
            fn get_data_res() -> Result<$t, DataStructureError> {
                let json = fs::read_to_string($fn_path().join($f)).unwrap();
                $t::decode_json(&json)
            }
            test_data_structure_read_data_set!();
            test_data_structure_verify_signature!();
            test_data_structure_verify_domain!();
        };
        ($suffix: ident, $t:ident, $f: literal, $fn_path: ident, $ignored: literal) => {
            paste! {
                fn [<get_data_res_ $suffix>]() -> Result<$t, DataStructureError> {
                    let json = fs::read_to_string($fn_path().join($f)).unwrap();
                    $t::decode_json(&json)
                }
                test_data_structure_read_data_set!($suffix);
                test_data_structure_verify_signature!($suffix, $ignored);
                test_data_structure_verify_domain!($suffix);
            }
        };
        ($suffix: ident, $t:ident, $f: literal, $fn_path: ident) => {
            paste! {
                fn [<get_data_res_ $suffix>]() -> Result<$t, DataStructureError> {
                    let json = fs::read_to_string($fn_path().join($f)).unwrap();
                    $t::decode_json(&json)
                }
                test_data_structure_read_data_set!($suffix);
                test_data_structure_verify_signature!($suffix);
                test_data_structure_verify_domain!($suffix);
            }
        };
    }
    pub(super) use test_data_structure;

    macro_rules! test_data_structure_read_data_set {
        () => {
            #[test]
            fn read_data_set() {
                let data_res = get_data_res();
                if data_res.is_err() {
                    println!("{:?}", data_res.as_ref().unwrap_err());
                }
                assert!(data_res.is_ok())
            }
        };
        ($suffix: ident) => {
            paste! {
                #[test]
                fn [<read_data_set_ $suffix>]() {
                    let data_res = [<get_data_res_ $suffix>]();
                    if data_res.is_err() {
                        println!("{:?}", data_res.as_ref().unwrap_err());
                    }
                    assert!(data_res.is_ok())
                }
            }
        };
    }
    pub(super) use test_data_structure_read_data_set;

    macro_rules! test_data_structure_verify_signature {
        ($ignored: literal) => {
            #[test]
            #[ignore = $ignored]
            fn verify_signature() {}
        };
        () => {
            #[test]
            fn verify_signature() {
                let data = get_data_res().unwrap();
                let ks = CONFIG_TEST.keystore().unwrap();
                let sign_validate_res = data.verify_signatures(&ks);
                for r in sign_validate_res {
                    if !r.is_ok() {
                        println!("error validating signature: {:?}", r.as_ref().unwrap_err())
                    }
                    assert!(r.is_ok());
                    assert!(r.unwrap())
                }
            }
        };
        ($suffix: ident, $ignored: literal) => {
            paste! {
                #[test]
                #[ignore = $ignored]
                fn [<verify_signature_ $suffix>]() {}
            }
        };
        ($suffix: ident) => {
            paste! {
                #[test]
                fn [<verify_signature_ $suffix>]() {
                    let data = [<get_data_res_ $suffix>]().unwrap();
                    let ks = CONFIG_TEST.keystore().unwrap();
                    let sign_validate_res = data.verify_signatures(&ks);
                    for r in sign_validate_res {
                        if !r.is_ok() {
                            println!("error validating signature: {:?}", r.as_ref().unwrap_err())
                        }
                        assert!(r.is_ok());
                        assert!(r.unwrap())
                    }
                }
            }
        };
    }
    pub(super) use test_data_structure_verify_signature;

    macro_rules! test_data_structure_verify_domain {
        () => {
            #[test]
            fn verify_domain() {
                let data = get_data_res().unwrap();
                let verifiy_domain_res = data.verifiy_domain();
                assert!(verifiy_domain_res.is_empty())
            }
        };
        ($suffix: ident) => {
            paste! {
                #[test]
                fn [<verify_domain_ $suffix>]() {
                    let data = [<get_data_res_ $suffix>]().unwrap();
                    let verifiy_domain_res = data.verifiy_domain();
                    assert!(verifiy_domain_res.is_empty())
                }
            }
        };
    }
    pub(super) use test_data_structure_verify_domain;
}
