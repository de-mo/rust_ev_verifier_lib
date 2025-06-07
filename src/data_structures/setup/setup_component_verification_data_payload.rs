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
        common_types::{CiphertextDef, EncryptionParametersDef, Signature},
        deserialize_seq_string_base64_to_seq_integer, implement_trait_verifier_data_json_decode,
        DataStructureError, VerifierDataDecode,
    },
    VerifierSetupDataType,
};
use crate::{
    data_structures::{VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{CertificateAuthority, VerifiySignatureTrait},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{elgamal::Ciphertext, Integer};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, ByteArray, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentVerificationDataPayload {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub partial_choice_return_codes_allow_list: Vec<String>,
    pub chunk_id: usize,
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub setup_component_verification_data: Vec<SetupComponentVerificationDataInner>,
    pub signature: Signature,
}

impl VerifierDataToTypeTrait for SetupComponentVerificationDataPayload {
    fn data_type() -> crate::data_structures::VerifierDataType {
        VerifierDataType::Setup(VerifierSetupDataType::SetupComponentVerificationDataPayload)
    }
}

implement_trait_verifier_data_json_decode!(SetupComponentVerificationDataPayload);

impl SetupComponentVerificationDataPayload {
    pub fn find_setup_component_verification_data_inner<'a>(
        &'a self,
        vc_id: &String,
    ) -> Option<&'a SetupComponentVerificationDataInner> {
        self.setup_component_verification_data
            .iter()
            .find(|d| &d.verification_card_id == vc_id)
    }

    pub fn verification_card_ids(&self) -> Vec<&String> {
        self.setup_component_verification_data
            .iter()
            .map(|d| &d.verification_card_id)
            .collect()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentVerificationDataInner {
    pub verification_card_id: String,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub verification_card_public_key: Vec<Integer>,
    #[serde(with = "CiphertextDef")]
    pub encrypted_hashed_squared_partial_choice_return_codes: Ciphertext,
    #[serde(with = "CiphertextDef")]
    pub encrypted_hashed_squared_confirmation_key: Ciphertext,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CombinedCorrectnessInformation {
    pub correctness_information_list: Vec<CorrectnessInformationElt>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CorrectnessInformationElt {
    pub correctness_id: String,
    pub number_of_selections: usize,
    pub number_of_voting_options: usize,
    pub list_of_write_in_options: Vec<usize>,
}

impl SetupComponentVerificationDataPayload {
    pub fn vc_ids(&self) -> Vec<&str> {
        self.setup_component_verification_data
            .iter()
            .map(|e| e.verification_card_id.as_str())
            .collect()
    }

    pub fn setup_component_verification_data(
        &self,
        vc_id: &str,
    ) -> Option<&SetupComponentVerificationDataInner> {
        self.setup_component_verification_data
            .iter()
            .find(|d| d.verification_card_id == vc_id)
    }
}

impl VerifyDomainTrait<String> for SetupComponentVerificationDataPayload {}

impl<'a> VerifiySignatureTrait<'a> for SetupComponentVerificationDataPayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("verification data"),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.verification_card_set_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmConfig)
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

impl<'a> From<&'a SetupComponentVerificationDataPayload> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentVerificationDataPayload) -> Self {
        let mut elts = vec![
            Self::from(&value.election_event_id),
            Self::from(&value.verification_card_set_id),
            Self::from(value.partial_choice_return_codes_allow_list.as_slice()),
            Self::from(&value.chunk_id),
            Self::from(&value.encryption_group),
        ];
        let l: Vec<HashableMessage> = value
            .setup_component_verification_data
            .iter()
            .map(Self::from)
            .collect();
        elts.push(Self::from(l));
        Self::from(elts)
    }
}

impl<'a> From<&'a SetupComponentVerificationDataInner> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentVerificationDataInner) -> Self {
        Self::from(vec![
            Self::from(&value.verification_card_id),
            Self::from(value.verification_card_public_key.as_slice()),
            Self::from(&value.encrypted_hashed_squared_partial_choice_return_codes),
            Self::from(&value.encrypted_hashed_squared_confirmation_key),
        ])
    }
}

impl<'a> From<&'a CombinedCorrectnessInformation> for HashableMessage<'a> {
    fn from(value: &'a CombinedCorrectnessInformation) -> Self {
        let l: Vec<HashableMessage> = value
            .correctness_information_list
            .iter()
            .map(Self::from)
            .collect();
        Self::from(l)
    }
}

impl<'a> From<&'a CorrectnessInformationElt> for HashableMessage<'a> {
    fn from(value: &'a CorrectnessInformationElt) -> Self {
        Self::from(vec![
            Self::from(&value.correctness_id),
            Self::from(&value.number_of_selections),
            Self::from(&value.number_of_voting_options),
            Self::from(value.list_of_write_in_options.as_slice()),
        ])
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
        SetupComponentVerificationDataPayload,
        "setupComponentVerificationDataPayload.0.json",
        test_setup_verification_card_set_path
    );
}
