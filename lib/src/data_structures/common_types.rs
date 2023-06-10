//! Type that are used in many structures

use super::{deserialize_seq_string_hex_to_seq_bigunit, deserialize_string_hex_to_bigunit};
use crypto_primitives::{
    byte_array::{ByteArray, Decode},
    hashing::HashableMessage,
};
use num_bigint::BigUint;
use serde::Deserialize;

/// Struct representing an encryption group
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct EncryptionGroup {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) p: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) q: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) g: BigUint,
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

impl From<&(BigUint, BigUint, BigUint)> for EncryptionGroup {
    fn from((p, q, g): &(BigUint, BigUint, BigUint)) -> Self {
        EncryptionGroup {
            p: p.clone(),
            q: q.clone(),
            g: g.clone(),
        }
    }
}

impl EncryptionGroup {
    pub(crate) fn as_tuple(&self) -> (&BigUint, &BigUint, &BigUint) {
        (&self.p, &self.q, &self.g)
    }
}

/// Struct representing the signature of a json file
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SignatureJson {
    pub(crate) signature_contents: String,
}

impl SignatureJson {
    /// Get the signature as ByteArray
    pub(crate) fn get_signature(&self) -> ByteArray {
        ByteArray::base64_decode(&self.signature_contents).unwrap()
    }
}

/// A proof (e,z) where the keys are _e and _z in json
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ProofUnderline {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_e")]
    pub(crate) e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "_z")]
    pub(crate) z: BigUint,
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
pub(crate) struct Proof {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) e: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) z: BigUint,
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

impl Proof {
    pub(crate) fn as_tuple(&self) -> (&BigUint, &BigUint) {
        (&self.e, &self.z)
    }
}

/// A proof (e,z) where the keys are _e and _z in json
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct DecryptionProof {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) e: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub(crate) z: Vec<BigUint>,
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
pub(crate) struct ExponentiatedEncryptedElement {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) gamma: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub(crate) phis: Vec<BigUint>,
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
