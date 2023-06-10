use super::super::{
    common_types::{EncryptionGroup, SignatureJson},
    deserialize_string_string_to_datetime, implement_trait_verifier_data_json_decode,
    VerifierDataDecode,
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ElectionEventContextPayload {
    pub(crate) encryption_group: EncryptionGroup,
    pub(crate) election_event_context: ElectionEventContext,
    pub(crate) signature: SignatureJson,
}

implement_trait_verifier_data_json_decode!(ElectionEventContextPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ElectionEventContext {
    pub(crate) election_event_id: String,
    //pub(crate) election_event_alias: String,
    //pub(crate) election_event_description: String,
    pub(crate) verification_card_set_contexts: Vec<VerificationCardSetContext>,
    #[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    pub(crate) start_time: NaiveDateTime,
    #[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    pub(crate) finish_time: NaiveDateTime,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[warn(dead_code)]
pub(crate) struct VerificationCardSetContext {
    pub(crate) verification_card_set_id: String,
    //pub(crate) verification_card_set_alias: String,
    //pub(crate) verification_card_set_description: String,
    pub(crate) ballot_box_id: String,
    //#[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    //pub(crate) ballot_box_start_time: NaiveDateTime,
    //#[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    //pub(crate) ballot_box_finish_time: NaiveDateTime,
    pub(crate) test_ballot_box: bool,
    pub(crate) number_of_write_in_fields: usize,
    pub(crate) number_of_voting_cards: usize,
    pub(crate) grace_period: usize,
    pub(crate) primes_mapping_table: PrimesMappingTable,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrimesMappingTable {
    pub(crate) p_table: Vec<PTableElement>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[warn(dead_code)]
pub(crate) struct PTableElement {
    pub(crate) actual_voting_option: String,
    pub(crate) encoded_voting_option: usize,
    //pub(crate) semantic_information: String,
}

impl VerificationCardSetContext {
    pub(crate) fn number_of_voters(&self) -> usize {
        self.number_of_voting_cards.clone()
    }

    pub(crate) fn number_of_voting_options(&self) -> usize {
        self.primes_mapping_table.p_table.len()
    }
}

impl ElectionEventContext {
    pub(crate) fn find_verification_card_set_context<'a>(
        &'a self,
        vcs_id: &String,
    ) -> Option<&'a VerificationCardSetContext> {
        self.verification_card_set_contexts
            .iter()
            .find(|c| &c.verification_card_set_id == vcs_id)
    }
}

impl<'a> From<&'a ElectionEventContextPayload> for HashableMessage<'a> {
    fn from(value: &'a ElectionEventContextPayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_context),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for ElectionEventContextPayload {
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("election event context"),
            HashableMessage::from(&self.election_event_context.election_event_id),
        ]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::SdmConfig
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

impl<'a> From<&'a ElectionEventContext> for HashableMessage<'a> {
    fn from(value: &'a ElectionEventContext) -> Self {
        let mut elts = vec![Self::from(&value.election_event_id)];
        let l: Vec<HashableMessage> = value
            .verification_card_set_contexts
            .iter()
            .map(|e| Self::from(e))
            .collect();
        elts.push(Self::from(l));
        elts.push(Self::from(&value.start_time));
        elts.push(Self::from(&value.finish_time));
        Self::from(elts)
    }
}

impl<'a> From<&'a VerificationCardSetContext> for HashableMessage<'a> {
    fn from(value: &'a VerificationCardSetContext) -> Self {
        let mut elts = vec![
            Self::from(&value.verification_card_set_id),
            Self::from(&value.ballot_box_id),
            Self::from(value.test_ballot_box),
            Self::from(&value.number_of_write_in_fields),
            Self::from(&value.number_of_voting_cards),
            Self::from(&value.grace_period),
        ];
        let l: Vec<HashableMessage> = value
            .primes_mapping_table
            .p_table
            .iter()
            .map(|e| Self::from(e))
            .collect();
        elts.push(Self::from(l));
        Self::from(elts)
    }
}

impl<'a> From<&'a PTableElement> for HashableMessage<'a> {
    fn from(value: &'a PTableElement) -> Self {
        Self::from(vec![
            Self::from(&value.actual_voting_option),
            Self::from(&value.encoded_voting_option),
        ])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::test::dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = dataset_tally_path()
            .join("setup")
            .join("electionEventContextPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ElectionEventContextPayload::from_json(&json);
        //println!("{:?}", r_eec.unwrap_err())
        assert!(r_eec.is_ok())
    }
}
