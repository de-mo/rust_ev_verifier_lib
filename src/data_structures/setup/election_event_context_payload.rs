use super::super::deserialize_string_hex_to_bigunit;
use super::encryption_parameters_payload::EncryptionGroup;
use num::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PTableElement {
    actual_voting_option: String,
    encoded_voting_option: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrimesMappingTable {
    p_table: Vec<PTableElement>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCardSetContext {
    verification_card_set_id: u32,
    ballot_box_id: u32,
    test_ballot_box: bool,
    number_of_write_in_fields: u32,
    number_of_voting_cards: u32,
    grace_period: u8,
    primes_mapping_table: PrimesMappingTable,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContext {
    election_event_id: u32,
    verification_card_set_contexts: Vec<VerificationCardSetContext>,
}

pub struct ElectionEventContextPayload {
    encryption_group: EncryptionGroup,
    election_event_context: ElectionEventContext,
}
