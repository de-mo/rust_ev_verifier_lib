use super::super::{
    common_types::{EncryptionParametersDef, Signature},
    deserialize_seq_seq_string_base64_to_seq_seq_integer,
    implement_trait_verifier_data_json_decode, DataStructureError, VerifierDataDecode,
};
use crate::{
    data_structures::{VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, ByteArray, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentTallyDataPayload {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub ballot_box_default_title: String,
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub verification_card_ids: Vec<String>,
    #[serde(deserialize_with = "deserialize_seq_seq_string_base64_to_seq_seq_integer")]
    pub verification_card_public_keys: Vec<Vec<Integer>>,
    pub signature: Signature,
}

impl VerifierDataToTypeTrait for SetupComponentTallyDataPayload {
    fn data_type() -> crate::data_structures::VerifierDataType {
        VerifierDataType::Context(super::VerifierContextDataType::SetupComponentTallyDataPayload)
    }
}

implement_trait_verifier_data_json_decode!(SetupComponentTallyDataPayload);

impl VerifyDomainTrait<String> for SetupComponentTallyDataPayload {}

impl<'a> From<&'a SetupComponentTallyDataPayload> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentTallyDataPayload) -> Self {
        let elts = vec![
            Self::from(&value.election_event_id),
            Self::from(&value.verification_card_set_id),
            Self::from(&value.ballot_box_default_title),
            Self::from(&value.encryption_group),
            Self::from(value.verification_card_ids.as_slice()),
            Self::from(value.verification_card_public_keys.as_slice()),
        ];
        Self::from(elts)
    }
}

impl<'a> VerifiySignatureTrait<'a> for SetupComponentTallyDataPayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Box<VerifySignatureError>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("tally data"),
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

#[cfg(test)]
mod test {
    use super::{
        super::super::test::{
            test_data_structure, test_data_structure_read_data_set,
            test_data_structure_verify_domain, test_data_structure_verify_signature,
        },
        *,
    };
    use crate::config::test::{test_context_verification_card_set_path, CONFIG_TEST};
    use std::fs;

    test_data_structure!(
        SetupComponentTallyDataPayload,
        "setupComponentTallyDataPayload.json",
        test_context_verification_card_set_path
    );
}
