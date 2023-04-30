use super::super::{
    common_types::{EncryptionGroup, SignatureJson},
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use crate::error::{create_verifier_error, VerifierError};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TallyComponentVotesPayload {
    pub election_event_id: String,
    pub ballot_id: String,
    pub ballot_box_id: String,
    pub encryption_group: EncryptionGroup,
    pub votes: Vec<Vec<usize>>,
    pub actual_selected_voting_options: Vec<Vec<String>>,
    pub decoded_write_in_votes: Vec<Vec<String>>,
    pub signature: SignatureJson,
}

implement_trait_verifier_data_json_decode!(TallyComponentVotesPayload);

#[cfg(test)]
mod test {
    use super::*;
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
            .join("tallyComponentVotesPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = TallyComponentVotesPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
