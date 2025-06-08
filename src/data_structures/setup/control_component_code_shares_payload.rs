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

use super::{
    super::{
        common_types::{CiphertextDef, EncryptionParametersDef, SchnorrProof, Signature},
        deserialize_seq_string_base64_to_seq_integer, implement_trait_verifier_data_json_decode,
        DataStructureError, VerifierDataDecode,DataStructureErrorImpl
    },
    VerifierSetupDataType,
};
use crate::{
    data_structures::{VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{CertificateAuthority, Keystore, VerifiySignatureTrait, VerifySignatureError},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{elgamal::Ciphertext, Integer};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, ByteArray, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ControlComponentCodeSharesPayload(pub Vec<ControlComponentCodeSharesPayloadInner>);

impl VerifierDataToTypeTrait for ControlComponentCodeSharesPayload {
    fn data_type() -> crate::data_structures::VerifierDataType {
        VerifierDataType::Setup(VerifierSetupDataType::ControlComponentCodeSharesPayload)
    }
}

implement_trait_verifier_data_json_decode!(ControlComponentCodeSharesPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeSharesPayloadInner {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub chunk_id: usize,
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub control_component_code_shares: Vec<ControlComponentCodeShare>,
    pub node_id: usize,
    pub signature: Signature,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeShare {
    pub verification_card_id: String,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub voter_choice_return_code_generation_public_key: Vec<Integer>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub voter_vote_cast_return_code_generation_public_key: Vec<Integer>,
    #[serde(with = "CiphertextDef")]
    pub exponentiated_encrypted_partial_choice_return_codes: Ciphertext,
    #[serde(with = "CiphertextDef")]
    pub exponentiated_encrypted_confirmation_key: Ciphertext,
    pub encrypted_partial_choice_return_code_exponentiation_proof: SchnorrProof,
    pub encrypted_confirmation_key_exponentiation_proof: SchnorrProof,
}

impl ControlComponentCodeSharesPayloadInner {
    pub fn vc_ids(&self) -> Vec<&str> {
        self.control_component_code_shares
            .iter()
            .map(|e| e.verification_card_id.as_str())
            .collect()
    }

    pub fn control_component_verification_data(
        &self,
        vc_id: &str,
    ) -> Option<&ControlComponentCodeShare> {
        self.control_component_code_shares
            .iter()
            .find(|d| d.verification_card_id == vc_id)
    }
}

impl VerifyDomainTrait<String> for ControlComponentCodeSharesPayloadInner {}

impl VerifyDomainTrait<String> for ControlComponentCodeSharesPayload {
    fn verifiy_domain(&self) -> Vec<String> {
        let mut errors: Vec<String> = self
            .0
            .iter()
            .enumerate()
            .filter(|(j, c)| j + 1 != c.node_id)
            .map(|(j, c)| {
                format!(
                    "The entry at position {} is not correspond to the node id {}",
                    j + 1,
                    c.node_id
                )
            })
            .collect();
        for (j, c) in self.0.iter().enumerate() {
            for error in c.verifiy_domain() {
                errors.push(format!("{} (node at position {})", error, j + 1))
            }
        }
        errors
    }
}

impl<'a> From<&'a ControlComponentCodeSharesPayloadInner> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentCodeSharesPayloadInner) -> Self {
        let mut elts = vec![
            Self::from(&value.election_event_id),
            Self::from(&value.verification_card_set_id),
            Self::from(&value.chunk_id),
            Self::from(&value.encryption_group),
        ];
        let l: Vec<HashableMessage> = value
            .control_component_code_shares
            .iter()
            .map(Self::from)
            .collect();
        elts.push(Self::from(l));
        elts.push(Self::from(&value.node_id));
        Self::from(elts)
    }
}

impl<'a> From<&'a ControlComponentCodeShare> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentCodeShare) -> Self {
        Self::from(vec![
            Self::from(&value.verification_card_id),
            Self::from(
                value
                    .voter_choice_return_code_generation_public_key
                    .as_slice(),
            ),
            Self::from(
                value
                    .voter_vote_cast_return_code_generation_public_key
                    .as_slice(),
            ),
            Self::from(&value.exponentiated_encrypted_partial_choice_return_codes),
            Self::from(&value.exponentiated_encrypted_confirmation_key),
            Self::from(&value.encrypted_partial_choice_return_code_exponentiation_proof),
            Self::from(&value.encrypted_confirmation_key_exponentiation_proof),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for ControlComponentCodeSharesPayloadInner {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("encrypted code shares"),
            HashableMessage::from(&self.node_id),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.verification_card_set_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        CertificateAuthority::get_ca_cc(&self.node_id)
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

impl<'a> VerifiySignatureTrait<'a> for ControlComponentCodeSharesPayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError> {
        unimplemented!()
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        unimplemented!()
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        unimplemented!()
    }

    fn get_signature(&self) -> ByteArray {
        unimplemented!()
    }

    fn verify_signatures(&'a self, keystore: &Keystore) -> Vec<Result<bool, VerifySignatureError>> {
        self.0
            .iter()
            .map(|e| e.verifiy_signature(keystore))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::test::{
            test_data_structure, test_data_structure_read_data_set,
            test_data_structure_verify_domain, test_data_structure_verify_signature,
        },
        *,
    };
    use crate::config::test::{test_setup_verification_card_set_path, CONFIG_TEST};
    use std::fs;

    test_data_structure!(
        ControlComponentCodeSharesPayload,
        "controlComponentCodeSharesPayload.0.json",
        test_setup_verification_card_set_path
    );
}
