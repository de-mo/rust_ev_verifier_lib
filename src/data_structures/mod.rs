//! Module to collect data structures of the verifier

pub mod common_types;
pub mod error;
pub mod setup;
pub mod tally;

use std::{io::BufRead, path::Path};

use self::{
    error::{DeserializeError, DeserializeErrorType},
    setup::{
        control_component_code_shares_payload::ControlComponentCodeSharesPayload,
        control_component_public_keys_payload::ControlComponentPublicKeysPayload,
        election_event_configuration::ElectionEventConfiguration,
        election_event_context_payload::ElectionEventContextPayload,
        encryption_parameters_payload::EncryptionParametersPayload,
        setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
        setup_component_tally_data_payload::SetupComponentTallyDataPayload,
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
use crate::{
    crypto_primitives::{
        byte_array::{ByteArray, Decode},
        num_bigint::Hexa,
    },
    error::{create_result_with_error, create_verifier_error, VerifierError},
    file_structure::{file::File, FileReadMode, FileType},
    setup_or_tally::SetupOrTally,
};
use num_bigint::BigUint;
use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
    Writer,
};
use roxmltree::Document;
use serde::de::{Deserialize, Deserializer, Error};

/// The type VerifierData implement an option between [VerifierSetupData] and [VerifierTallyData]
pub type VerifierData = SetupOrTally<VerifierSetupData, VerifierTallyData>;

/// The type VerifierDataType implement an option between [VerifierSetupDataType] and [VerifierTallyDataType]
pub type VerifierDataType = SetupOrTally<VerifierSetupDataType, VerifierTallyDataType>;

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

/// Trait implementing the collection of the specific setup data type from the enum object
pub trait VerifierSetupDataTrait {
    /// Get the EncryptionParametersPayload is the enum is from correct type. Else give None
    fn encryption_parameters_payload(&self) -> Option<&EncryptionParametersPayload> {
        None
    }
    fn setup_component_public_keys_payload(&self) -> Option<&SetupComponentPublicKeysPayload> {
        None
    }
    fn election_event_context_payload(&self) -> Option<&ElectionEventContextPayload> {
        None
    }
    fn setup_component_tally_data_payload(&self) -> Option<&SetupComponentTallyDataPayload> {
        None
    }
    fn control_component_public_keys_payload(&self) -> Option<&ControlComponentPublicKeysPayload> {
        None
    }
    fn setup_component_verification_data_payload(
        &self,
    ) -> Option<&SetupComponentVerificationDataPayload> {
        None
    }
    fn control_component_code_shares_payload(&self) -> Option<&ControlComponentCodeSharesPayload> {
        None
    }
    fn election_event_configuration(&self) -> Option<&ElectionEventConfiguration> {
        None
    }
}

/// Trait implementing the collection of the specific tally data type from the enum object
pub trait VerifierTallyDataTrait {
    fn e_voting_decrypt(&self) -> Option<&EVotingDecrypt> {
        None
    }
    fn ech_0110(&self) -> Option<&ECH0110> {
        None
    }
    fn ech_0222(&self) -> Option<&ECH0222> {
        None
    }
    fn tally_component_votes_payload(&self) -> Option<&TallyComponentVotesPayload> {
        None
    }
    fn tally_component_shuffle_payload(&self) -> Option<&TallyComponentShufflePayload> {
        None
    }
    fn control_component_ballot_box_payload(&self) -> Option<&ControlComponentBallotBoxPayload> {
        None
    }

    fn control_component_shuffle_payload(&self) -> Option<&ControlComponentShufflePayload> {
        None
    }
}

/// A trait defining the necessary function to decode to the Verifier Data
pub trait VerifierDataDecode: Sized {
    fn from_file(f: &File, t: &FileType, mode: &FileReadMode) -> Result<Self, DeserializeError> {
        match mode {
            FileReadMode::Memory => Self::from_file_memory(f, t),
            FileReadMode::Streaming => Self::from_file_stream(f, t),
        }
    }

    fn from_file_memory(f: &File, t: &FileType) -> Result<Self, DeserializeError> {
        let s = f.read_data().map_err(|e| {
            create_verifier_error!(
                DeserializeErrorType::FileError,
                format!("Error reading data in file {}", f.to_str()),
                e
            )
        })?;
        match t {
            FileType::Json => Self::from_json(&s),
            FileType::Xml => {
                let doc = Document::parse(&s).map_err(|e| {
                    create_verifier_error!(
                        DeserializeErrorType::XMLError,
                        "Cannot parse content of xml file",
                        e
                    )
                })?;
                Self::from_roxmltree(&doc)
            }
        }
    }

    fn from_file_stream(f: &File, t: &FileType) -> Result<Self, DeserializeError> {
        match t {
            FileType::Json => create_result_with_error!(
                DeserializeErrorType::JSONError,
                "from_file not implemented for JSON Files"
            ),
            FileType::Xml => Self::from_xml_file(&f.get_path()),
        }
    }

    fn from_json(_: &String) -> Result<Self, DeserializeError> {
        create_result_with_error!(
            DeserializeErrorType::JSONError,
            "from_json not implemented now"
        )
    }

    fn from_roxmltree<'a>(_: &'a Document<'a>) -> Result<Self, DeserializeError> {
        create_result_with_error!(
            DeserializeErrorType::JSONError,
            "from_roxmltree not implemented"
        )
    }

    fn from_xml_file(_: &Path) -> Result<Self, DeserializeError> {
        create_result_with_error!(
            DeserializeErrorType::JSONError,
            "from_xml_file not implemented"
        )
    }
}

/// Macro to automatically implement the DataStructureTrait for a type
macro_rules! implement_trait_verifier_data_json_decode {
    ($s: ty) => {
        impl VerifierDataDecode for $s {
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
use implement_trait_verifier_data_json_decode;

impl VerifierSetupDataTrait for VerifierData {
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

    fn election_event_configuration(&self) -> Option<&ElectionEventConfiguration> {
        match self {
            VerifierData::Setup(d) => d.election_event_configuration(),
            VerifierData::Tally(_) => None,
        }
    }
}

impl VerifierTallyDataTrait for VerifierData {
    fn e_voting_decrypt(&self) -> Option<&EVotingDecrypt> {
        match self {
            VerifierData::Setup(_) => None,
            VerifierData::Tally(d) => d.e_voting_decrypt(),
        }
    }
    fn ech_0110(&self) -> Option<&ECH0110> {
        match self {
            VerifierData::Setup(_) => None,
            VerifierData::Tally(d) => d.ech_0110(),
        }
    }
    fn ech_0222(&self) -> Option<&ECH0222> {
        match self {
            VerifierData::Setup(_) => None,
            VerifierData::Tally(d) => d.ech_0222(),
        }
    }
    fn tally_component_votes_payload(&self) -> Option<&TallyComponentVotesPayload> {
        match self {
            VerifierData::Setup(_) => None,
            VerifierData::Tally(d) => d.tally_component_votes_payload(),
        }
    }
    fn tally_component_shuffle_payload(&self) -> Option<&TallyComponentShufflePayload> {
        match self {
            VerifierData::Setup(_) => None,
            VerifierData::Tally(d) => d.tally_component_shuffle_payload(),
        }
    }
    fn control_component_ballot_box_payload(&self) -> Option<&ControlComponentBallotBoxPayload> {
        match self {
            VerifierData::Setup(_) => None,
            VerifierData::Tally(d) => d.control_component_ballot_box_payload(),
        }
    }
    fn control_component_shuffle_payload(&self) -> Option<&ControlComponentShufflePayload> {
        match self {
            VerifierData::Setup(_) => None,
            VerifierData::Tally(d) => d.control_component_shuffle_payload(),
        }
    }
}

impl VerifierDataType {
    /// Read VerifierDataType from a String as JSON
    pub fn verifier_data_from_file(&self, f: &File) -> Result<VerifierData, DeserializeError> {
        match self {
            VerifierDataType::Setup(t) => {
                t.verifier_data_from_file(f).map(|r| VerifierData::Setup(r))
            }
            VerifierDataType::Tally(t) => {
                t.verifier_data_from_file(f).map(|r| VerifierData::Tally(r))
            }
        }
    }
}

// reads from a start tag all the way to the corresponding end tag,
// returns the bytes of the whole tag
pub fn xml_read_to_end_into_buffer<R: BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
    junk_buf: &mut Vec<u8>,
) -> Result<Vec<u8>, DeserializeError> {
    let mut depth = 0;
    let mut output_buf: Vec<u8> = Vec::new();
    let mut w = Writer::new(&mut output_buf);
    let tag_name = start_tag.name();
    w.write_event(Event::Start(start_tag.clone()))
        .map_err(|e| {
            create_verifier_error!(
                DeserializeErrorType::XMLError,
                format!("Error writing event {:?} in writer", start_tag),
                e
            )
        })?;
    loop {
        junk_buf.clear();
        let event = reader.read_event_into(junk_buf).map_err(|e| {
            create_verifier_error!(DeserializeErrorType::XMLError, "Error reading event", e)
        })?;
        w.write_event(&event).map_err(|e| {
            create_verifier_error!(
                DeserializeErrorType::XMLError,
                format!("Error writing event {:?} in writer", event),
                e
            )
        })?;

        match event {
            Event::Start(e) if e.name() == tag_name => depth += 1,
            Event::End(e) if e.name() == tag_name => {
                if depth == 0 {
                    return Ok(output_buf);
                }
                depth -= 1;
            }
            Event::Eof => {
                panic!("oh no")
            }
            _ => {}
        }
    }
}

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
