use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, SignatureJson},
    deserialize_seq_string_hex_to_seq_bigunit, implement_trait_verifier_data_json_decode,
    VerifierDataDecode,
};
use anyhow::anyhow;
use crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetupComponentVerificationDataPayload {
    pub(crate) election_event_id: String,
    pub(crate) verification_card_set_id: String,
    pub(crate) partial_choice_return_codes_allow_list: Vec<String>,
    pub(crate) chunk_id: usize,
    pub(crate) encryption_group: EncryptionGroup,
    pub(crate) setup_component_verification_data: Vec<SetupComponentVerificationDataInner>,
    pub(crate) combined_correctness_information: CombinedCorrectnessInformation,
    pub(crate) signature: SignatureJson,
}

implement_trait_verifier_data_json_decode!(SetupComponentVerificationDataPayload);

impl SetupComponentVerificationDataPayload {
    #[allow(dead_code)]
    pub(crate) fn find_setup_component_verification_data_inner<'a>(
        &'a self,
        vc_id: &String,
    ) -> Option<&'a SetupComponentVerificationDataInner> {
        self.setup_component_verification_data
            .iter()
            .find(|d| &d.verification_card_id == vc_id)
    }

    pub(crate) fn verification_card_ids<'a>(&'a self) -> Vec<&'a String> {
        self.setup_component_verification_data
            .iter()
            .map(|d| &d.verification_card_id)
            .collect()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetupComponentVerificationDataInner {
    pub(crate) verification_card_id: String,
    pub(crate) encrypted_hashed_squared_confirmation_key: ExponentiatedEncryptedElement,
    pub(crate) encrypted_hashed_squared_partial_choice_return_codes: ExponentiatedEncryptedElement,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub(crate) verification_card_public_key: Vec<BigUint>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CombinedCorrectnessInformation {
    pub(crate) correctness_information_list: Vec<CorrectnessInformationElt>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CorrectnessInformationElt {
    pub(crate) correctness_id: String,
    pub(crate) number_of_selections: usize,
    pub(crate) number_of_voting_options: usize,
    pub(crate) list_of_write_in_options: Vec<usize>,
}

impl<'a> VerifiySignatureTrait<'a> for SetupComponentVerificationDataPayload {
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("verification data"),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.verification_card_set_id),
        ]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::SdmConfig
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

impl<'a> From<&'a SetupComponentVerificationDataPayload> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentVerificationDataPayload) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.election_event_id));
        elts.push(Self::from(&value.verification_card_set_id));
        elts.push(Self::from(&value.partial_choice_return_codes_allow_list));
        elts.push(Self::from(&value.chunk_id));
        elts.push(Self::from(&value.encryption_group));
        let l: Vec<HashableMessage> = value
            .setup_component_verification_data
            .iter()
            .map(|e| Self::from(e))
            .collect();
        elts.push(Self::from(l));
        elts.push(Self::from(&value.combined_correctness_information));
        Self::from(elts)
    }
}

impl<'a> From<&'a SetupComponentVerificationDataInner> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentVerificationDataInner) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.verification_card_id));
        elts.push(Self::from(&value.encrypted_hashed_squared_confirmation_key));
        elts.push(Self::from(
            &value.encrypted_hashed_squared_partial_choice_return_codes,
        ));
        elts.push(Self::from(&value.verification_card_public_key));
        Self::from(elts)
    }
}

impl<'a> From<&'a CombinedCorrectnessInformation> for HashableMessage<'a> {
    fn from(value: &'a CombinedCorrectnessInformation) -> Self {
        let l: Vec<HashableMessage> = value
            .correctness_information_list
            .iter()
            .map(|e| Self::from(e))
            .collect();
        Self::from(l)
    }
}

impl<'a> From<&'a CorrectnessInformationElt> for HashableMessage<'a> {
    fn from(value: &'a CorrectnessInformationElt) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.correctness_id));
        elts.push(Self::from(&value.number_of_selections));
        elts.push(Self::from(&value.number_of_voting_options));
        elts.push(Self::from(&value.list_of_write_in_options));
        Self::from(elts)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::test::dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = dataset_tally_path()
            .join("setup")
            .join("verification_card_sets")
            .join("681B3488DE4CD4AD7FCED14B7A654169")
            .join("setupComponentVerificationDataPayload.0.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = SetupComponentVerificationDataPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
