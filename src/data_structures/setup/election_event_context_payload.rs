use super::super::{
    common_types::{EncryptionGroup, Signature},
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_data_structure, DataStructureTrait,
};
use crate::error::{create_verifier_error, VerifierError};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContextPayload {
    pub encryption_group: EncryptionGroup,
    pub election_event_context: ElectionEventContext,
    pub signature: Signature,
}

implement_trait_data_structure!(ElectionEventContextPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PTableElement {
    pub actual_voting_option: String,
    pub encoded_voting_option: usize,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrimesMappingTable {
    pub p_table: Vec<PTableElement>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCardSetContext {
    pub verification_card_set_id: String,
    pub ballot_box_id: String,
    pub test_ballot_box: bool,
    pub number_of_write_in_fields: usize,
    pub number_of_voting_cards: usize,
    pub grace_period: usize,
    pub primes_mapping_table: PrimesMappingTable,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContext {
    pub election_event_id: String,
    pub verification_card_set_contexts: Vec<VerificationCardSetContext>,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup")
            .join("electionEventContextPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ElectionEventContextPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
