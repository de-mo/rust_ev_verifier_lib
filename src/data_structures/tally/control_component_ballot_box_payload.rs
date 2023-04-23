use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, SignatureJson},
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_verifier_data_decode, VerifierDataDecode,
};
use crate::{
    data_structures::common_types::{DecryptionProof, Proof},
    error::{create_verifier_error, VerifierError},
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentBallotBoxPayload {
    pub encryption_group: EncryptionGroup,
    pub election_event_id: String,
    pub ballot_box_id: String,
    pub node_id: usize,
    pub confirmed_encrypted_votes: Vec<ConfirmedEncryptedVote>,
    pub signature: SignatureJson,
}
implement_trait_verifier_data_decode!(ControlComponentBallotBoxPayload);

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
    use crate::file_structure::FileType;
    use std::fs;
    use std::path::Path;

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset1")
            .join("tally")
            .join("ballot_boxes")
            .join("9a19164550794441b25f7f744f2e91fb")
            .join("controlComponentBallotBoxPayload_1.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ControlComponentBallotBoxPayload::from_string(&json, &FileType::Json);
        println!("{:?}", r_eec.as_ref().err());
        assert!(r_eec.is_ok())
    }
}
