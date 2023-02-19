use super::super::Signature;
use super::encryption_parameters_payload::EncryptionGroup;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PTableElement {
    actual_voting_option: String,
    encoded_voting_option: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrimesMappingTable {
    p_table: Vec<PTableElement>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCardSetContext {
    verification_card_set_id: String,
    ballot_box_id: String,
    test_ballot_box: bool,
    number_of_write_in_fields: usize,
    number_of_voting_cards: usize,
    grace_period: usize,
    primes_mapping_table: PrimesMappingTable,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContext {
    election_event_id: String,
    verification_card_set_contexts: Vec<VerificationCardSetContext>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContextPayload {
    encryption_group: EncryptionGroup,
    election_event_context: ElectionEventContext,
    signature: Signature,
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
        let r_eec: Result<ElectionEventContextPayload, serde_json::Error> =
            serde_json::from_str(&json);
        assert!(r_eec.is_ok())
    }
}
