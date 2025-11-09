// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

//! Module to define the structure of the data and to read the data from the files into these structures
//!
//! The module is separate in two module: [setup] and [tally]
//!
pub mod common_types;
pub mod context;
pub mod dataset;
mod serde;
pub mod tally;
mod xml;

#[cfg(test)]
pub use xml::mock;

pub use self::{
    context::{
        VerifierContextDataType, election_event_context_payload::ElectionEventContextPayload,
    },
    dataset::DatasetType,
    tally::{
        VerifierTallyDataType,
        control_component_ballot_box_payload::ControlComponentBallotBoxPayload,
        control_component_shuffle_payload::ControlComponentShufflePayload,
        tally_component_shuffle_payload::TallyComponentShufflePayload,
    },
};
use crate::config::VerifierConfig;
use serde::*;

use roxmltree::Error as RoXmlTreeError;
use std::{path::Path, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error(transparent)]
pub struct DataStructureError(#[from] DataStructureErrorImpl);

#[derive(Error, Debug, Clone)]
enum DataStructureErrorImpl {
    #[error("Not implemented: {0}")]
    NotImplemented(&'static str),
    #[error("Error parsing json {msg}")]
    ParseJSON {
        msg: String,
        source: Arc<serde_json::Error>,
    },
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    ParseRoXML { msg: String, source: RoXmlTreeError },
}

/// The type VerifierDataType implement an option between
/// [VerifierContextDataType] and [VerifierTallyDataType]
pub type VerifierDataType = DatasetType<VerifierContextDataType, VerifierTallyDataType>;

/// Trait to add the funcitonality to get the [VerifierDataType] from the verifier data
pub trait VerifierDataToTypeTrait {
    fn data_type() -> VerifierDataType;
}

/// A trait defining the necessary function to decode to the Verifier Data
pub trait VerifierDataDecode: Sized {
    /// Decode the data from a json string
    ///
    /// # Return
    /// The decoded data or [DataStructureError] if something wrong, e.g. if it is not allowed, or if an error
    /// occured during the decoding
    fn decode_json(_: &str) -> Result<Self, DataStructureError> {
        Err(DataStructureError::from(
            DataStructureErrorImpl::NotImplemented("decode_json"),
        ))
    }

    /// Decode the data from a xml string
    ///
    /// # Return
    /// The decoded data or [DataStructureError] if something wrong, e.g. if it is not allowed, or if an error
    /// occured during the decoding
    fn decode_xml(_: String) -> Result<Self, DataStructureError> {
        Err(DataStructureError::from(
            DataStructureErrorImpl::NotImplemented("decode_xml"),
        ))
    }

    /// Prepare the streamin of data from a json file
    ///
    /// # Return
    /// The decoded data or [DataStructureError] if something wrong, e.g. if it is not allowed, or if an error
    /// occured during the decoding
    fn stream_json(_: &Path) -> Result<Self, DataStructureError> {
        Err(DataStructureError::from(
            DataStructureErrorImpl::NotImplemented("stream_json"),
        ))
    }

    /// Prepare the streamin of data from a xml file
    ///
    /// # Return
    /// The decoded data or [DataStructureError] if something wrong, e.g. if it is not allowed, or if an error
    /// occured during the decoding
    fn stream_xml(_: &Path) -> Result<Self, DataStructureError> {
        Err(DataStructureError::from(
            DataStructureErrorImpl::NotImplemented("stream_xml"),
        ))
    }
}

/// Macro to automatically implement the DataStructureTrait for a type
macro_rules! implement_trait_verifier_data_json_decode {
    ($s: ty) => {
        impl VerifierDataDecode for $s {
            fn decode_json(s: &str) -> Result<Self, DataStructureError> {
                serde_json::from_str(s)
                    .map_err(|e| DataStructureErrorImpl::ParseJSON {
                        msg: format!("Cannot deserialize json"),
                        source: Arc::new(e),
                    })
                    .map_err(DataStructureError::from)
            }
        }
    };
}
use implement_trait_verifier_data_json_decode;

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

    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{HashableMessage, Integer};
    use serde::de::DeserializeOwned;
    use serde_json::Value;
    use std::path::Path;

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
                assert!(data_res.is_ok(), "{:?}", data_res.as_ref().unwrap_err())
            }
        };
        ($suffix: ident) => {
            paste! {
                #[test]
                fn [<read_data_set_ $suffix>]() {
                    let data_res = [<get_data_res_ $suffix>]();
                    assert!(data_res.is_ok(), "{:?}", data_res.as_ref().unwrap_err())
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
                let ks = get_keystore();
                let sign_validate_res = data.verify_signatures(&ks);
                for r in sign_validate_res {
                    assert!(
                        r.is_ok(),
                        "error validating signature: {:?}",
                        r.as_ref().unwrap_err()
                    );
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
                    let ks = get_keystore();
                    let sign_validate_res = data.verify_signatures(&ks);
                    for r in sign_validate_res {
                        assert!(r.is_ok(), "error validating signature: {:?}", r.as_ref().unwrap_err());
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
                let verifiy_domain_res = data.verifiy_domain(&EmptyContext::default());
                assert!(verifiy_domain_res.is_empty())
            }
        };
        ($suffix: ident) => {
            paste! {
                #[test]
                fn [<verify_domain_ $suffix>]() {
                    let data = [<get_data_res_ $suffix>]().unwrap();
                    let verifiy_domain_res = data.verifiy_domain(&EmptyContext::default());
                    assert!(verifiy_domain_res.is_empty())
                }
            }
        };
    }
    pub(super) use test_data_structure_verify_domain;

    pub fn json_to_hashable_message<'a>(value: &'a Value) -> HashableMessage<'a> {
        match value {
            v if v.is_array() => HashableMessage::from(
                value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|e| json_to_hashable_message(e))
                    .collect::<Vec<_>>(),
            ),
            v if v.is_boolean() => HashableMessage::from(value.as_bool().unwrap()),
            v if v.is_number() => {
                HashableMessage::from(Integer::from_str_radix(&value.to_string(), 10).unwrap())
            }
            v if v.is_string() => HashableMessage::from(value.as_str().unwrap()),
            _ => panic!("Not possible"),
        }
    }

    #[derive(Debug, Clone)]
    pub struct OutputVerifySignature {
        pub h: Value,
        pub d: String,
    }
    #[derive(Debug, Clone)]
    pub struct TestDataStructureVerifySignature<T>
    where
        T: DeserializeOwned,
    {
        pub description: String,
        pub context: T,
        pub output: OutputVerifySignature,
    }

    pub fn json_to_testdata<T>(v: &Value) -> TestDataStructureVerifySignature<T>
    where
        T: DeserializeOwned,
    {
        TestDataStructureVerifySignature {
            description: v["description"].as_str().unwrap().to_string(),
            context: serde_json::from_value::<T>(v["input"].clone()).unwrap(),
            output: OutputVerifySignature {
                h: v["output"]["h"].clone(),
                d: v["output"]["d"].as_str().unwrap().to_string(),
            },
        }
    }

    pub fn file_to_test_cases(path: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    macro_rules! test_hash_json {
        ($t: ident, $p: literal) => {
            #[test]
            fn test_hash_json() {
                let path = test_data_signature_hash_path().join($p);
                for tc in file_to_test_cases(&path).as_array().unwrap().iter() {
                    let test_data = json_to_testdata::<$t>(tc);
                    let hash_context = HashableMessage::from(&test_data.context);
                    let h = json_to_hashable_message(&test_data.output.h);
                    let comp = hash_context.compare_to(&h, None);
                    assert!(
                        comp.is_ok(),
                        "{}: {}",
                        test_data.description,
                        comp.unwrap_err()
                    );
                    let hashed = hash_context.recursive_hash();
                    assert!(
                        hashed.is_ok(),
                        "{}: {}",
                        test_data.description,
                        hashed.unwrap_err()
                    );
                    assert_eq!(
                        hashed.unwrap().base64_encode().unwrap(),
                        test_data.output.d,
                        "{}",
                        test_data.description
                    )
                }
            }
        };
    }
    pub(super) use test_hash_json;
}
