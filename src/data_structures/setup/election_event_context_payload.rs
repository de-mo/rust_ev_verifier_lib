use super::super::{
    common_types::{Signature, EncryptionParametersDef}, deserialize_string_string_to_datetime,
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use rust_ev_crypto_primitives::{
    ByteArray, EncryptionParameters, HashableMessage,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContextPayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_context: ElectionEventContext,
    pub signature: Signature,
}

implement_trait_verifier_data_json_decode!(ElectionEventContextPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContext {
    pub election_event_id: String,
    //pub election_event_alias: String,
    //pub election_event_description: String,
    pub verification_card_set_contexts: Vec<VerificationCardSetContext>,
    #[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    pub start_time: NaiveDateTime,
    #[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    pub finish_time: NaiveDateTime,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[warn(dead_code)]
pub struct VerificationCardSetContext {
    pub verification_card_set_id: String,
    //pub verification_card_set_alias: String,
    //pub verification_card_set_description: String,
    pub ballot_box_id: String,
    //#[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    //pub ballot_box_start_time: NaiveDateTime,
    //#[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    //pub ballot_box_finish_time: NaiveDateTime,
    pub test_ballot_box: bool,
    pub number_of_write_in_fields: usize,
    pub number_of_voting_cards: usize,
    pub grace_period: usize,
    pub primes_mapping_table: PrimesMappingTable,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrimesMappingTable {
    pub p_table: Vec<PTableElement>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[warn(dead_code)]
pub struct PTableElement {
    pub actual_voting_option: String,
    pub encoded_voting_option: usize,
    //pub semantic_information: String,
}

impl VerificationCardSetContext {
    pub fn number_of_voters(&self) -> usize {
        self.number_of_voting_cards
    }

    pub fn number_of_voting_options(&self) -> usize {
        self.primes_mapping_table.p_table.len()
    }
}

impl ElectionEventContext {
    pub fn find_verification_card_set_context<'a>(
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

    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("election event context"),
            HashableMessage::from(&self.election_event_context.election_event_id),
        ]
    }

    fn get_certificate_authority(&self) -> anyhow::Result<String> {
        Ok(String::from(CertificateAuthority::SdmConfig))
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
            .map(Self::from)
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
            .map(Self::from)
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
    use crate::config::test::test_dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = test_dataset_tally_path()
            .join("setup")
            .join("electionEventContextPayload.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = ElectionEventContextPayload::from_json(&json);
        //println!("{:?}", r_eec.unwrap_err())
        assert!(r_eec.is_ok())
    }
}
