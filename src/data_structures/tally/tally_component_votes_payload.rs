use super::super::{
    common_types::{EncryptionParametersDef, Signature},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use anyhow::anyhow;
use rust_ev_crypto_primitives::EncryptionParameters;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TallyComponentVotesPayload {
    pub election_event_id: String,
    pub ballot_id: String,
    pub ballot_box_id: String,
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub votes: Vec<Vec<usize>>,
    pub actual_selected_voting_options: Vec<Vec<String>>,
    pub decoded_write_in_votes: Vec<Vec<String>>,
    pub signature: Signature,
}

implement_trait_verifier_data_json_decode!(TallyComponentVotesPayload);

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_ballot_box_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = test_ballot_box_path().join("tallyComponentVotesPayload.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = TallyComponentVotesPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
