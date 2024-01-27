//! Type that are used in many structures

use super::{
    deserialize_seq_string_hex_to_seq_bigunit, deserialize_string_hex_to_bigunit, CheckDomainTrait,
};
use num_bigint::BigUint;
use rust_ev_crypto_primitives::{
    check_g, check_p, check_q, ByteArray, Decode, EncryptionParameters, HashableMessage,
};
use serde::Deserialize;

/// Struct representing an encryption group
#[derive(Deserialize, Debug, Clone)]
#[serde(remote = "EncryptionParameters")]
pub struct EncryptionParametersDef {
    #[serde(
        deserialize_with = "deserialize_string_hex_to_bigunit",
        getter = "EncryptionParameters::p"
    )]
    pub p: BigUint,
    #[serde(
        deserialize_with = "deserialize_string_hex_to_bigunit",
        getter = "EncryptionParameters::q"
    )]
    pub q: BigUint,
    #[serde(
        deserialize_with = "deserialize_string_hex_to_bigunit",
        getter = "EncryptionParameters::g"
    )]
    pub g: BigUint,
}

impl From<EncryptionParametersDef> for EncryptionParameters {
    fn from(def: EncryptionParametersDef) -> Self {
        Self::from((&def.p, &def.q, &def.g))
    }
}

impl CheckDomainTrait for EncryptionParameters {
    fn check_domain(&self) -> Vec<anyhow::Error> {
        let mut res = vec![];
        if let Some(e) = check_p(self.p()) {
            res.push(anyhow::anyhow!(e))
        }
        if let Some(e) = check_q(self.p(), self.q()) {
            res.push(anyhow::anyhow!(e))
        }
        if let Some(e) = check_g(self.p(), self.g()) {
            res.push(anyhow::anyhow!(e))
        }
        res
    }
}

/// Struct representing the signature of a json file
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub signature_contents: String,
}

impl Signature {
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
        Self::from(vec![Self::from(&(value.e)), Self::from(&(value.z))])
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
        Self::from(vec![Self::from(&(value.e)), Self::from(&(value.z))])
    }
}

impl Proof {
    pub fn as_tuple(&self) -> (&BigUint, &BigUint) {
        (&self.e, &self.z)
    }
}

/// A decryption proof (e,z) where the keys are _e and _z in json
#[derive(Deserialize, Debug, Clone)]
pub struct DecryptionProof {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub e: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub z: Vec<BigUint>,
}

impl<'a> From<&'a DecryptionProof> for HashableMessage<'a> {
    fn from(value: &'a DecryptionProof) -> Self {
        Self::from(vec![Self::from(&(value.e)), Self::from(&(value.z))])
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
