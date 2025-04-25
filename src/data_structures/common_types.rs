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

//! Type that are used in many structures

use super::{deserialize_seq_string_base64_to_seq_integer, deserialize_string_base64_to_integer};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::elgamal::Ciphertext;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, ByteArray, DecodeTrait, HashableMessage,
};
use serde::Deserialize;

/// Struct representing an encryption group
#[derive(Deserialize, Debug, Clone)]
#[serde(remote = "EncryptionParameters")]
pub struct EncryptionParametersDef {
    #[serde(
        deserialize_with = "deserialize_string_base64_to_integer",
        getter = "EncryptionParameters::p"
    )]
    pub p: Integer,
    #[serde(
        deserialize_with = "deserialize_string_base64_to_integer",
        getter = "EncryptionParameters::q"
    )]
    pub q: Integer,
    #[serde(
        deserialize_with = "deserialize_string_base64_to_integer",
        getter = "EncryptionParameters::g"
    )]
    pub g: Integer,
}

impl From<EncryptionParametersDef> for EncryptionParameters {
    fn from(def: EncryptionParametersDef) -> Self {
        Self::from((&def.p, &def.q, &def.g))
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

/// A proof (e,z) where the keys are e and z in json
#[derive(Deserialize, Debug, Clone)]
pub struct SchnorrProof {
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    #[serde(alias = "_e")]
    pub e: Integer,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    #[serde(alias = "_z")]
    pub z: Integer,
}

impl<'a> From<&'a SchnorrProof> for HashableMessage<'a> {
    fn from(value: &'a SchnorrProof) -> Self {
        Self::from(vec![Self::from(&(value.e)), Self::from(&(value.z))])
    }
}

impl SchnorrProof {
    pub fn as_tuple(&self) -> (&Integer, &Integer) {
        (&self.e, &self.z)
    }
}

/// A decryption proof (e,z) where the keys are e and z (array) in json
#[derive(Deserialize, Debug, Clone)]
pub struct DecryptionProof {
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub e: Integer,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub z: Vec<Integer>,
}

impl<'a> From<&'a DecryptionProof> for HashableMessage<'a> {
    fn from(value: &'a DecryptionProof) -> Self {
        Self::from(vec![Self::from(&(value.e)), Self::from(value.z.as_slice())])
    }
}

impl DecryptionProof {
    pub fn as_tuple(&self) -> (&Integer, &[Integer]) {
        (&self.e, self.z.as_slice())
    }
}

/// A exponentieted encrypted element (gamman, phi)
#[derive(Deserialize, Debug, Clone)]
#[serde(remote = "Ciphertext")]
pub struct CiphertextDef {
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub gamma: Integer,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub phis: Vec<Integer>,
}

impl<'a> From<&'a CiphertextDef> for HashableMessage<'a> {
    fn from(value: &'a CiphertextDef) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&(value.gamma)));
        for p in value.phis.iter() {
            elts.push(Self::from(p));
        }
        Self::from(elts)
    }
}

impl From<CiphertextDef> for Ciphertext {
    fn from(value: CiphertextDef) -> Self {
        Ciphertext {
            gamma: value.gamma,
            phis: value.phis,
        }
    }
}
