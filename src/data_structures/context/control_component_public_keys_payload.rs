use super::super::{
    common_types::{EncryptionParametersDef, ProofUnderline, Signature},
    deserialize_seq_string_base64_to_seq_integer, implement_trait_verifier_data_json_decode,
    DataStructureError, VerifierDataDecode,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};
use anyhow::anyhow;
use rust_ev_crypto_primitives::Integer;
use rust_ev_crypto_primitives::{
    ByteArray, EncryptionParameters, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentPublicKeysPayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub control_component_public_keys: ControlComponentPublicKeys,
    pub signature: Signature,
}

implement_trait_verifier_data_json_decode!(ControlComponentPublicKeysPayload);

impl VerifyDomainTrait<anyhow::Error> for ControlComponentPublicKeysPayload {}

impl<'a> From<&'a ControlComponentPublicKeysPayload> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentPublicKeysPayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.control_component_public_keys),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for ControlComponentPublicKeysPayload {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("OnlineCC keys"),
            HashableMessage::from(&self.control_component_public_keys.node_id),
            HashableMessage::from(&self.election_event_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        CertificateAuthority::get_ca_cc(&self.control_component_public_keys.node_id)
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentPublicKeys {
    pub node_id: usize,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub ccrj_choice_return_codes_encryption_public_key: Vec<Integer>,
    pub ccrj_schnorr_proofs: Vec<ProofUnderline>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub ccmj_election_public_key: Vec<Integer>,
    pub ccmj_schnorr_proofs: Vec<ProofUnderline>,
}

impl<'a> From<&'a ControlComponentPublicKeys> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentPublicKeys) -> Self {
        let mut elts = vec![
            Self::from(&value.node_id),
            Self::from(&value.ccrj_choice_return_codes_encryption_public_key),
        ];
        let ccrj: Vec<HashableMessage> = value.ccrj_schnorr_proofs.iter().map(Self::from).collect();
        elts.push(Self::from(ccrj));
        elts.push(Self::from(&value.ccmj_election_public_key));
        let ccmj: Vec<HashableMessage> = value.ccmj_schnorr_proofs.iter().map(Self::from).collect();
        elts.push(Self::from(ccmj));
        Self::from(elts)
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
    use crate::config::test::{test_datasets_context_path, CONFIG_TEST};
    use std::fs;

    test_data_structure!(
        ControlComponentPublicKeysPayload,
        "controlComponentPublicKeysPayload.1.json",
        test_datasets_context_path
    );
}
