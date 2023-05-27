//! Type that are used in many structures

use super::{deserialize_seq_string_hex_to_seq_bigunit, deserialize_string_hex_to_bigunit};
use crate::crypto_primitives::{
    byte_array::{ByteArray, Decode},
    hashing::HashableMessage,
};
use num_bigint::BigUint;
use serde::Deserialize;

/// Struct representing an encryption group
#[derive(Deserialize, Debug, Clone)]
pub struct EncryptionGroup {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub p: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub q: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub g: BigUint,
}

impl<'a> From<&'a EncryptionGroup> for HashableMessage<'a> {
    fn from(value: &'a EncryptionGroup) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.p));
        elts.push(Self::from(&value.q));
        elts.push(Self::from(&value.g));
        Self::from(elts)
    }
}

/// Struct representing the signature of a json file
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignatureJson {
    pub signature_contents: String,
}

impl SignatureJson {
    /// Get the signature as ByteArray
    pub fn get_signature(&self) -> ByteArray {
        ByteArray::base64_decode(&self.signature_contents).unwrap()
    }
}

/// A proof (e,z) where the keys are _e and _z in json
#[derive(Deserialize, Debug, Clone)]
pub struct ProofUnderline {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_e")]
    pub e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_z")]
    pub z: BigUint,
}

impl<'a> From<&'a ProofUnderline> for HashableMessage<'a> {
    fn from(value: &'a ProofUnderline) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&(value.e)));
        elts.push(Self::from(&(value.z)));
        Self::from(elts)
    }
}

/// A proof (e,z) where the keys are e and z in json
#[derive(Deserialize, Debug, Clone)]
pub struct Proof {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub z: BigUint,
}

/// The possibility to transorm a [ProofUnderline] to [Proof]
impl From<&ProofUnderline> for Proof {
    fn from(value: &ProofUnderline) -> Self {
        Proof {
            e: value.e.clone(),
            z: value.z.clone(),
        }
    }
}

impl<'a> From<&'a Proof> for HashableMessage<'a> {
    fn from(value: &'a Proof) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&(value.e)));
        elts.push(Self::from(&(value.z)));
        Self::from(elts)
    }
}

/// A proof (e,z) where the keys are _e and _z in json
#[derive(Deserialize, Debug, Clone)]
pub struct DecryptionProof {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub e: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub z: Vec<BigUint>,
}

impl<'a> From<&'a DecryptionProof> for HashableMessage<'a> {
    fn from(value: &'a DecryptionProof) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&(value.e)));
        elts.push(Self::from(&(value.z)));
        Self::from(elts)
    }
}

/// A exponentieted encrypted element (gamman, phi)
#[derive(Deserialize, Debug, Clone)]
pub struct ExponentiatedEncryptedElement {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub gamma: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub phis: Vec<BigUint>,
}

impl<'a> From<&'a ExponentiatedEncryptedElement> for HashableMessage<'a> {
    fn from(value: &'a ExponentiatedEncryptedElement) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&(value.gamma)));
        for p in value.phis.iter() {
            elts.push(Self::from(p));
        }
        Self::from(elts)
    }
}
