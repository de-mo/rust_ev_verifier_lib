use super::super::{
    common_types::{EncryptionParametersDef, Signature},
    deserialize_string_string_to_datetime, implement_trait_verifier_data_json_decode,
    VerifierDataDecode,
};
use crate::config::Config as VerifierConfig;
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};
use anyhow::anyhow;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use regex::Regex;
use rust_ev_crypto_primitives::{
    ByteArray, EncryptionParameters, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContextPayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub seed: String,
    pub small_primes: Vec<usize>,
    pub election_event_context: ElectionEventContext,
    pub signature: Signature,
}

implement_trait_verifier_data_json_decode!(ElectionEventContextPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectionEventContext {
    pub election_event_id: String,
    pub election_event_alias: String,
    pub election_event_description: String,
    pub verification_card_set_contexts: Vec<VerificationCardSetContext>,
    #[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    pub start_time: NaiveDateTime,
    #[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    pub finish_time: NaiveDateTime,
    pub maximum_number_of_voting_options: usize,
    pub maximum_number_of_selections: usize,
    pub maximum_number_of_write_ins_plus_one: usize,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[warn(dead_code)]
pub struct VerificationCardSetContext {
    pub verification_card_set_id: String,
    pub verification_card_set_alias: String,
    pub verification_card_set_description: String,
    pub ballot_box_id: String,
    #[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    pub ballot_box_start_time: NaiveDateTime,
    #[serde(deserialize_with = "deserialize_string_string_to_datetime")]
    pub ballot_box_finish_time: NaiveDateTime,
    pub test_ballot_box: bool,
    pub number_of_voting_cards: usize,
    pub grace_period: usize,
    pub primes_mapping_table: PrimesMappingTable,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrimesMappingTable {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub p_table: Vec<PTableElement>,
    pub number_of_voting_options: usize,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[warn(dead_code)]
pub struct PTableElement {
    pub actual_voting_option: String,
    pub encoded_voting_option: usize,
    pub semantic_information: String,
    pub correctness_information: String,
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

fn validate_seed(seed: &str) -> Vec<anyhow::Error> {
    let mut res = vec![];
    if seed.len() != 16 {
        return vec![anyhow!(format!(
            "The seed {} must be of size 16, actual ist {}",
            seed,
            seed.len(),
        ))];
    }
    let re = Regex::new(r"[A-Z]{2}_\d{8}_(TT|TP|PP)\d{2}").unwrap();
    if !re.is_match(seed) {
        return vec![anyhow!(format!(
            "The seed {} does not match the format  CT_YYYYMMDD_XYnm",
            seed,
        ))];
    }
    let date = seed.get(3..11).unwrap();
    if let Err(e) = NaiveDate::parse_from_str(date, "%Y%m%d") {
        res.push(anyhow!(format!(
            "the date {} of the seed {} is not valid: {}",
            seed, date, e
        )))
    }
    let event_type = seed.get(12..14).unwrap();
    if event_type != "TT" && event_type != "TP" && event_type != "PP" {
        res.push(anyhow!(format!(
            "the event type {} of the seed {} is not valid. Must be TT, TP or PP",
            seed, event_type
        )))
    }
    res
}

impl VerifyDomainTrait for ElectionEventContextPayload {
    fn verifiy_domain(&self) -> Vec<anyhow::Error> {
        let mut res = self.encryption_group.verifiy_domain();
        // For 05.01 (seed)
        res.extend(validate_seed(&self.seed));
        // For 5.02
        if !self.small_primes.len()
            == VerifierConfig::maximum_number_of_supported_voting_options_n_sup()
        {
            res.push(
                anyhow!(
                    format!(
                        "The list of small primes {} is not equal to the maximal number of voting options {}",
                        self.small_primes.len(),
                        VerifierConfig::maximum_number_of_supported_voting_options_n_sup()
                    )
                )
            );
        }
        // for 5.02
        let mut sp = self.small_primes.clone();
        sp.sort();
        if sp != self.small_primes {
            res.push(anyhow!("Small primes list is not in ascending order"));
        }
        // for 5.02
        if sp[0] < 5 {
            res.push(anyhow!(
                "The small primes contain 2 or 3, what is not allowed"
            ));
        }
        res
    }
}

impl<'a> From<&'a ElectionEventContextPayload> for HashableMessage<'a> {
    fn from(value: &'a ElectionEventContextPayload) -> Self {
        let sp_hash: Vec<HashableMessage> = value
            .small_primes
            .iter()
            .map(HashableMessage::from)
            .collect();
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.seed),
            Self::from(sp_hash),
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
        elts.push(Self::from(&value.election_event_alias));
        elts.push(Self::from(&value.election_event_description));
        let l: Vec<HashableMessage> = value
            .verification_card_set_contexts
            .iter()
            .map(Self::from)
            .collect();
        elts.push(Self::from(l));
        elts.push(Self::from(&value.start_time));
        elts.push(Self::from(&value.finish_time));
        elts.push(Self::from(&value.maximum_number_of_voting_options));
        elts.push(Self::from(&value.maximum_number_of_selections));
        elts.push(Self::from(&value.maximum_number_of_write_ins_plus_one));
        Self::from(elts)
    }
}

impl<'a> From<&'a VerificationCardSetContext> for HashableMessage<'a> {
    fn from(value: &'a VerificationCardSetContext) -> Self {
        Self::from(vec![
            Self::from(&value.verification_card_set_id),
            Self::from(&value.verification_card_set_alias),
            Self::from(&value.verification_card_set_description),
            Self::from(&value.ballot_box_id),
            Self::from(&value.ballot_box_start_time),
            Self::from(&value.ballot_box_finish_time),
            Self::from(value.test_ballot_box),
            Self::from(&value.number_of_voting_cards),
            Self::from(&value.grace_period),
            Self::from(&value.primes_mapping_table),
        ])
    }
}

impl<'a> From<&'a PrimesMappingTable> for HashableMessage<'a> {
    fn from(value: &'a PrimesMappingTable) -> Self {
        let l: Vec<HashableMessage> = value.p_table.iter().map(Self::from).collect();
        Self::from(vec![Self::from(&value.encryption_group), Self::from(l)])
    }
}

impl<'a> From<&'a PTableElement> for HashableMessage<'a> {
    fn from(value: &'a PTableElement) -> Self {
        Self::from(vec![
            Self::from(&value.actual_voting_option),
            Self::from(&value.encoded_voting_option),
            Self::from(&value.semantic_information),
            Self::from(&value.correctness_information),
        ])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_datasets_context_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = test_datasets_context_path().join("electionEventContextPayload.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = ElectionEventContextPayload::from_json(&json);
        if r_eec.is_err() {
            println!("{:?}", r_eec.as_ref().unwrap_err());
        }
        assert!(r_eec.is_ok())
    }

    #[test]
    fn read_validate_seed() {
        assert!(validate_seed("SG_20230101_TT01").is_empty());
        assert!(!validate_seed("SG_20230101_TT0").is_empty());
        assert!(!validate_seed("Sg_20230101_TT01").is_empty());
        assert!(!validate_seed("SG_202301a1_TT01").is_empty());
        assert!(!validate_seed("SG_20230101_tt01").is_empty());
        assert!(!validate_seed("SG_20230101_TT0a").is_empty());
        assert!(!validate_seed("SG_20231301_TT01").is_empty());
        assert!(!validate_seed("SG_20231201_AA01").is_empty());
    }
}
