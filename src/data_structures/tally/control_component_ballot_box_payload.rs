use super::super::{
    common_types::{ExponentiatedEncryptedElement, Signature, EncryptionParametersDef},
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
    use crate::config::test::test_dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = test_dataset_tally_path()
            .join("tally")
            .join("ballot_boxes")
            .join("4AB4F95B8114C1DFEDB9586ADBFE36B3")
            .join("controlComponentBallotBoxPayload_1.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = ControlComponentBallotBoxPayload::from_json(&json);
        println!("{:?}", r_eec.as_ref().err());
        assert!(r_eec.is_ok())
    }
}
