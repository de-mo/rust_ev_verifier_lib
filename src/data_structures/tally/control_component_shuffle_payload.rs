use super::super::{
    common_types::{EncryptionParametersDef, ExponentiatedEncryptedElement, Signature},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use super::tally_component_shuffle_payload::VerifiableShuffle;
use crate::{
    data_structures::common_types::DecryptionProof,
    direct_trust::{CertificateAuthority, VerifiySignatureTrait},
};
use anyhow::anyhow;
use rust_ev_crypto_primitives::{
    ByteArray, EncryptionParameters, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentShufflePayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub ballot_box_id: String,
    pub node_id: usize,
    pub verifiable_shuffle: VerifiableShuffle,
    pub verifiable_decryptions: VerifiableDecryptions,
    pub signature: Signature,
}
implement_trait_verifier_data_json_decode!(ControlComponentShufflePayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiableDecryptions {
    pub ciphertexts: Vec<ExponentiatedEncryptedElement>,
    pub decryption_proofs: Vec<DecryptionProof>,
}

impl VerifyDomainTrait<anyhow::Error> for ControlComponentShufflePayload {}

impl<'a> From<&'a ControlComponentShufflePayload> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentShufflePayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_box_id),
            Self::from(&value.node_id),
            Self::from(&value.verifiable_shuffle),
            Self::from(&value.verifiable_decryptions),
        ])
    }
}

impl<'a> From<&'a VerifiableDecryptions> for HashableMessage<'a> {
    fn from(value: &'a VerifiableDecryptions) -> Self {
        Self::from(vec![
            Self::from(
                value
                    .ciphertexts
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
            Self::from(
                value
                    .decryption_proofs
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for ControlComponentShufflePayload {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("shuffle"),
            HashableMessage::from(&self.node_id),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.ballot_box_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        CertificateAuthority::get_ca_cc(&self.node_id)
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
    use crate::config::test::{test_ballot_box_path, CONFIG_TEST};
    use std::fs;

    test_data_structure!(
        ControlComponentShufflePayload,
        "controlComponentShufflePayload_1.json",
        test_ballot_box_path
    );
}
