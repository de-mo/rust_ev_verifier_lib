use super::super::{
    common_types::{CiphertextDef, EncryptionParametersDef, Signature},
    implement_trait_verifier_data_json_decode, DataStructureError, VerifierDataDecode,
};
use crate::{
    data_structures::common_types::{DecryptionProof, SchnorrProof},
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError},
};

use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::{Ciphertext, EncryptionParameters},
    ByteArray, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentBallotBoxPayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub ballot_box_id: String,
    pub node_id: usize,
    pub confirmed_encrypted_votes: Vec<ConfirmedEncryptedVote>,
    pub signature: Signature,
}
implement_trait_verifier_data_json_decode!(ControlComponentBallotBoxPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedEncryptedVote {
    pub context_ids: ContextIds,
    #[serde(with = "CiphertextDef")]
    pub encrypted_vote: Ciphertext,
    #[serde(with = "CiphertextDef")]
    pub exponentiated_encrypted_vote: Ciphertext,
    #[serde(with = "CiphertextDef")]
    pub encrypted_partial_choice_return_codes: Ciphertext,
    pub exponentiation_proof: SchnorrProof,
    pub plaintext_equality_proof: DecryptionProof,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextIds {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub verification_card_id: String,
}

impl VerifyDomainTrait<String> for ControlComponentBallotBoxPayload {}

impl<'a> From<&'a ControlComponentBallotBoxPayload> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentBallotBoxPayload) -> Self {
        let votes: Vec<Self> = value
            .confirmed_encrypted_votes
            .iter()
            .map(Self::from)
            .collect();
        let mut res = vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_box_id),
            Self::from(&value.node_id),
        ];
        if !votes.is_empty() {
            res.push(Self::from(votes))
        }
        Self::from(res)
    }
}

impl<'a> From<&'a ConfirmedEncryptedVote> for HashableMessage<'a> {
    fn from(value: &'a ConfirmedEncryptedVote) -> Self {
        Self::from(vec![
            Self::from(&value.context_ids),
            Self::from(&value.encrypted_vote),
            Self::from(&value.exponentiated_encrypted_vote),
            Self::from(&value.encrypted_partial_choice_return_codes),
            Self::from(&value.exponentiation_proof),
            Self::from(&value.plaintext_equality_proof),
        ])
    }
}

impl<'a> From<&'a ContextIds> for HashableMessage<'a> {
    fn from(value: &'a ContextIds) -> Self {
        Self::from(vec![
            Self::from(&value.election_event_id),
            Self::from(&value.verification_card_set_id),
            Self::from(&value.verification_card_id),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for ControlComponentBallotBoxPayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Box<VerifySignatureError>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("ballotbox"),
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
    use crate::config::test::{
        test_ballot_box_one_vote_path, test_ballot_box_zero_vote_path, CONFIG_TEST,
    };
    use std::fs;

    test_data_structure!(
        ControlComponentBallotBoxPayload,
        "controlComponentBallotBoxPayload_1.json",
        test_ballot_box_one_vote_path
    );

    #[test]
    fn test_signature_empty_votes() {
        let json = fs::read_to_string(
            test_ballot_box_zero_vote_path().join("controlComponentBallotBoxPayload_4.json"),
        )
        .unwrap();
        let data = ControlComponentBallotBoxPayload::decode_json(&json).unwrap();
        let ks = CONFIG_TEST.keystore().unwrap();
        let sign_validate_res = data.verify_signatures(&ks);
        for r in sign_validate_res {
            assert!(r.is_ok());
            assert!(r.unwrap())
        }
    }
}
