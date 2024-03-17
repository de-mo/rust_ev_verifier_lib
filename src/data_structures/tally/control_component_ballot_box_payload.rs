use super::super::{
    common_types::{EncryptionParametersDef, ExponentiatedEncryptedElement, Signature},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use crate::data_structures::common_types::{DecryptionProof, Proof};
use anyhow::anyhow;
use rust_ev_crypto_primitives::EncryptionParameters;
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
    pub encrypted_vote: ExponentiatedEncryptedElement,
    pub exponentiated_encrypted_vote: ExponentiatedEncryptedElement,
    pub encrypted_partial_choice_return_codes: ExponentiatedEncryptedElement,
    pub exponentiation_proof: Proof,
    pub plaintext_equality_proof: DecryptionProof,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextIds {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub verification_card_id: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_ballot_box_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = test_ballot_box_path().join("controlComponentBallotBoxPayload_1.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = ControlComponentBallotBoxPayload::from_json(&json);
        println!("{:?}", r_eec.as_ref().err());
        assert!(r_eec.is_ok())
    }
}
