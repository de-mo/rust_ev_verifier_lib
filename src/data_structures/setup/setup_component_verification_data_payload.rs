use super::super::{
    common_types::{EncryptionParametersDef, ExponentiatedEncryptedElement, Signature},
    deserialize_seq_string_base64_to_seq_integer, implement_trait_verifier_data_json_decode,
    DataStructureError, VerifierDataDecode,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError};
use rust_ev_crypto_primitives::Integer;
use rust_ev_crypto_primitives::{
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
    pub encrypted_hashed_squared_partial_choice_return_codes: ExponentiatedEncryptedElement,
    pub encrypted_hashed_squared_confirmation_key: ExponentiatedEncryptedElement,
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
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Box<VerifySignatureError>> {
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
            Self::from(&value.partial_choice_return_codes_allow_list),
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
            Self::from(&value.verification_card_public_key),
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
            Self::from(&value.list_of_write_in_options),
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
