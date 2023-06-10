use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, SignatureJson},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use crate::data_structures::common_types::{DecryptionProof, Proof};
use anyhow::anyhow;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ControlComponentBallotBoxPayload {
    pub(crate) encryption_group: EncryptionGroup,
    pub(crate) election_event_id: String,
    pub(crate) ballot_box_id: String,
    pub(crate) node_id: usize,
    pub(crate) confirmed_encrypted_votes: Vec<ConfirmedEncryptedVote>,
    pub(crate) signature: SignatureJson,
}
implement_trait_verifier_data_json_decode!(ControlComponentBallotBoxPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfirmedEncryptedVote {
    pub(crate) context_ids: ContextIds,
    pub(crate) encrypted_vote: ExponentiatedEncryptedElement,
    pub(crate) exponentiated_encrypted_vote: ExponentiatedEncryptedElement,
    pub(crate) encrypted_partial_choice_return_codes: ExponentiatedEncryptedElement,
    pub(crate) exponentiation_proof: Proof,
    pub(crate) plaintext_equality_proof: DecryptionProof,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContextIds {
    pub(crate) election_event_id: String,
    pub(crate) verification_card_set_id: String,
    pub(crate) verification_card_id: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::test::dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = dataset_tally_path()
            .join("tally")
            .join("ballot_boxes")
            .join("4AB4F95B8114C1DFEDB9586ADBFE36B3")
            .join("controlComponentBallotBoxPayload_1.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ControlComponentBallotBoxPayload::from_json(&json);
        println!("{:?}", r_eec.as_ref().err());
        assert!(r_eec.is_ok())
    }
}
