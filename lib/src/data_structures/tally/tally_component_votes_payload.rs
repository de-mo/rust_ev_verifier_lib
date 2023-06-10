use super::super::{
    common_types::{EncryptionGroup, SignatureJson},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use anyhow::anyhow;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TallyComponentVotesPayload {
    pub(crate) election_event_id: String,
    pub(crate) ballot_id: String,
    pub(crate) ballot_box_id: String,
    pub(crate) encryption_group: EncryptionGroup,
    pub(crate) votes: Vec<Vec<usize>>,
    pub(crate) actual_selected_voting_options: Vec<Vec<String>>,
    pub(crate) decoded_write_in_votes: Vec<Vec<String>>,
    pub(crate) signature: SignatureJson,
}

implement_trait_verifier_data_json_decode!(TallyComponentVotesPayload);

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
            .join("tallyComponentVotesPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = TallyComponentVotesPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
