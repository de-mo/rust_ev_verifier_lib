use super::super::{
    common_types::{EncryptionParametersDef, Signature},
    deserialize_string_string_to_datetime, implement_trait_verifier_data_json_decode,
    DataStructureError, VerifierDataDecode,
};
use crate::{
    config::VerifierConfig,
    data_structures::{verifiy_domain_length_unique_id, VerifierDataType},
};
use crate::{
    data_structures::VerifierDataToTypeTrait,
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError},
};
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use regex::Regex;
use rust_ev_system_library::preliminaries::{
    GetHashElectionEventContextContext, PTableElement,
    VerificationCardSetContext as VerificationCardSetContextInSystemLibrary,
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, ByteArray, DomainVerifications, HashableMessage,
    RecursiveHashTrait, VerifyDomainTrait,
};
use serde::de::{Deserializer, Error as SerdeError};
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

impl VerifierDataToTypeTrait for ElectionEventContextPayload {
    fn data_type() -> crate::data_structures::VerifierDataType {
        VerifierDataType::Context(super::VerifierContextDataType::ElectionEventContextPayload)
    }
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
    #[serde(deserialize_with = "deserialize_p_table")]
    pub p_table: Vec<PTableElement>,
    pub number_of_voting_options: usize,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[warn(dead_code)]
pub struct PTableElementDef {
    pub actual_voting_option: String,
    pub encoded_voting_option: usize,
    pub semantic_information: String,
    pub correctness_information: String,
}

impl From<PTableElementDef> for PTableElement {
    fn from(def: PTableElementDef) -> Self {
        Self {
            actual_voting_option: def.actual_voting_option,
            encoded_voting_option: def.encoded_voting_option,
            semantic_information: def.semantic_information,
            correctness_information: def.correctness_information,
        }
    }
}

fn deserialize_p_table<'de, D>(deserializer: D) -> Result<Vec<PTableElement>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<PTableElement>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element())? {
                let r_b = PTableElement::from(
                    serde_json::from_value::<PTableElementDef>(v).map_err(A::Error::custom)?,
                );
                vec.push(r_b);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
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
    /// Find the verification card set context with the id
    ///
    /// Return None if not found
    pub fn find_verification_card_set_context<'a>(
        &'a self,
        vcs_id: &str,
    ) -> Option<&'a VerificationCardSetContext> {
        self.verification_card_set_contexts
            .iter()
            .find(|c| c.verification_card_set_id == vcs_id)
    }

    /// Find the verification card set context with the ballot box id
    ///
    /// Return None if not found
    pub fn find_verification_card_set_context_with_bb_id<'a>(
        &'a self,
        bb_id: &str,
    ) -> Option<&'a VerificationCardSetContext> {
        self.verification_card_set_contexts
            .iter()
            .find(|c| c.ballot_box_id == bb_id)
    }

    /// Get the list of all verification card set ids
    pub fn vcs_ids(&self) -> Vec<&str> {
        self.verification_card_set_contexts
            .iter()
            .map(|c| c.verification_card_set_id.as_str())
            .collect()
    }

    /// Get the list of all ballot box ids
    pub fn bb_ids(&self) -> Vec<&str> {
        self.verification_card_set_contexts
            .iter()
            .map(|c| c.ballot_box_id.as_str())
            .collect()
    }

    /// Find the id of the ballot box using the id of the verification card set
    ///
    /// Return None if not found
    pub fn find_ballot_box_id<'a>(&'a self, vcs_id: &str) -> Option<&'a str> {
        self.find_verification_card_set_context(vcs_id)
            .map(|c| c.ballot_box_id.as_str())
    }
}

/// Validate seed according to the specifications of Swiss Post
///
/// seed = <Canton>_<Date>_<TT|TP|PP>_nn
fn validate_seed(seed: &str) -> Vec<String> {
    let mut res = vec![];
    if seed.len() != 16 {
        return vec![format!(
            "The seed {} must be of size 16, actual ist {}",
            seed,
            seed.len(),
        )];
    }
    let re = Regex::new(r"[A-Z]{2}_\d{8}_(TT|TP|PP)\d{2}").unwrap();
    if !re.is_match(seed) {
        return vec![format!(
            "The seed {} does not match the format  CT_YYYYMMDD_XYnm",
            seed,
        )];
    }
    let date = seed.get(3..11).unwrap();
    if let Err(e) = NaiveDate::parse_from_str(date, "%Y%m%d") {
        res.push(format!(
            "the date {} of the seed {} is not valid: {}",
            seed, date, e
        ))
    }
    let event_type = seed.get(12..14).unwrap();
    if event_type != "TT" && event_type != "TP" && event_type != "PP" {
        res.push(format!(
            "the event type {} of the seed {} is not valid. Must be TT, TP or PP",
            seed, event_type
        ))
    }
    res
}

/// Validate small primes are correct
///
/// - Size is equal to the max. supported voting options
/// - Is sorted correctly (for 05.02)
/// - The first ist greater or equal than 5 (for 05.02)
fn validate_small_primes(small_primes: &[usize]) -> Vec<String> {
    let mut res = vec![];
    // Len is correct
    if !small_primes.len() == VerifierConfig::maximum_number_of_supported_voting_options_n_sup() {
        res.push(format!(
            "The list of small primes {} is not equal to the maximal number of voting options {}",
            small_primes.len(),
            VerifierConfig::maximum_number_of_supported_voting_options_n_sup()
        ));
    }
    // is sorted
    if !small_primes.windows(2).all(|p| p[0] < p[1]) {
        res.push("Small primes list is not in ascending order".to_string());
    }
    // for 5.02
    if small_primes[0] < 5 {
        res.push("The small primes contain 2 or 3, what is not allowed".to_string());
    };
    res
}

/// Validate if the number of voting option in the verification card set context is correct
///
/// - The number of voting options expected is the same than counted
/// - The number is greater than 0 and less than the maximum supported voting options (for 05.03)
fn validate_voting_options_number(p_table: &PrimesMappingTable) -> Vec<String> {
    let mut res = vec![];
    // Value number_of_voting_options is correct
    if p_table.number_of_voting_options != p_table.p_table.len() {
        res.push(format!(
            "The  number of voting options expected {} is not the same that the number of voting options listed {}",
            p_table.number_of_voting_options,
            p_table.p_table.len()
        ));
    }
    // number of voting options must be greater that 0
    if p_table.number_of_voting_options == 0 {
        res.push("The  number of voting options must be greater than 0".to_string());
    }
    // number of voting options must be smaller or equal than max. supported voting options
    if p_table.number_of_voting_options
        > VerifierConfig::maximum_number_of_supported_voting_options_n_sup()
    {
        res.push(format!(
            "The  number of voting options expected {} must be smaller or equal the the max. supported voting options {}",
            p_table.number_of_voting_options,
            VerifierConfig::maximum_number_of_supported_voting_options_n_sup()
        ));
    }
    res
}

impl VerifyDomainTrait<String> for ElectionEventContextPayload {
    fn new_domain_verifications() -> DomainVerifications<Self, String> {
        let mut res = DomainVerifications::default();
        res.add_verification(|v: &Self| {
            v.encryption_group
                .verifiy_domain()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
        });
        res.add_verification(|v: &Self| validate_seed(&v.seed));
        res.add_verification(|v: &Self| validate_small_primes(&v.small_primes));
        res.add_verification_with_vec_of_vec_errors(|v| {
            v.election_event_context
                .verification_card_set_contexts
                .iter()
                .map(|c| validate_voting_options_number(&c.primes_mapping_table))
                .collect()
        });
        res.add_verification(|v| {
            verifiy_domain_length_unique_id(
                &v.election_event_context.election_event_id,
                "election event id",
            )
        });
        res
    }
}

impl VerifyDomainTrait<String> for VerificationCardSetContext {
    fn new_domain_verifications() -> DomainVerifications<Self, String> {
        let mut res = DomainVerifications::default();
        res.add_verification(|v: &Self| validate_voting_options_number(&v.primes_mapping_table));
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
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Box<VerifySignatureError>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("election event context"),
            HashableMessage::from(&self.election_event_context.election_event_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmConfig)
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

impl<'a> From<&'a ElectionEventContext> for GetHashElectionEventContextContext<'a, 'a> {
    fn from(value: &'a ElectionEventContext) -> Self {
        Self {
            encryption_parameters: &value.verification_card_set_contexts[0]
                .primes_mapping_table
                .encryption_group,
            ee: &value.election_event_id,
            ee_alias: &value.election_event_alias,
            ee_descr: &value.election_event_description,
            vcs_contexts: value
                .verification_card_set_contexts
                .iter()
                .map(VerificationCardSetContextInSystemLibrary::from)
                .collect::<Vec<_>>(),
            t_s_ee: &value.start_time,
            t_f_ee: &value.finish_time,
            n_max: value.maximum_number_of_voting_options,
            psi_max: value.maximum_number_of_selections,
            delta_max: value.maximum_number_of_write_ins_plus_one,
        }
    }
}

impl<'a> From<&'a ElectionEventContext> for HashableMessage<'a> {
    fn from(value: &'a ElectionEventContext) -> Self {
        let temp = GetHashElectionEventContextContext::from(value);
        let hashed = HashableMessage::from(&temp).recursive_hash().unwrap();
        HashableMessage::Hashed(hashed)
    }
}

impl<'a> From<&'a VerificationCardSetContext> for VerificationCardSetContextInSystemLibrary<'a> {
    fn from(value: &'a VerificationCardSetContext) -> Self {
        Self {
            vcs: &value.verification_card_set_id,
            vcs_alias: &value.verification_card_set_alias,
            vcs_desc: &value.verification_card_set_description,
            bb: &value.ballot_box_id,
            t_s_bb: &value.ballot_box_start_time,
            t_f_bb: &value.ballot_box_finish_time,
            test_ballot_box: value.test_ballot_box,
            upper_n_upper_e: value.number_of_voting_cards,
            grace_period: value.grace_period,
            p_table: &value.primes_mapping_table.p_table,
            encryption_parameters: &value.primes_mapping_table.encryption_group,
        }
    }
}

#[cfg(test)]
mod test {
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
        EncodeTrait, RecursiveHashTrait,
    };

    use super::{
        super::super::test::{
            test_data_structure, test_data_structure_read_data_set,
            test_data_structure_verify_domain, test_data_structure_verify_signature,
        },
        *,
    };
    use crate::config::test::{test_datasets_context_path, test_resources_path, CONFIG_TEST};
    use std::fs;

    test_data_structure!(
        ElectionEventContextPayload,
        "electionEventContextPayload.json",
        test_datasets_context_path
    );

    #[test]
    fn test_validate_seed() {
        assert!(validate_seed("SG_20230101_TT01").is_empty());
        assert!(!validate_seed("SG_20230101_TT0").is_empty());
        assert!(!validate_seed("Sg_20230101_TT01").is_empty());
        assert!(!validate_seed("SG_202301a1_TT01").is_empty());
        assert!(!validate_seed("SG_20230101_tt01").is_empty());
        assert!(!validate_seed("SG_20230101_TT0a").is_empty());
        assert!(!validate_seed("SG_20231301_TT01").is_empty());
        assert!(!validate_seed("SG_20231201_AA01").is_empty());
    }

    #[test]
    fn error_number_of_voting_options() {
        let mut ee = get_data_res().unwrap();
        ee.election_event_context.verification_card_set_contexts[0]
            .primes_mapping_table
            .number_of_voting_options = 1;
        assert!(!ee.verifiy_domain().is_empty());
    }

    #[test]
    fn error_election_event_id() {
        let mut ee = get_data_res().unwrap();
        ee.election_event_context.election_event_id = "1234345".to_string();
        assert!(!ee.verifiy_domain().is_empty());
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        d: String,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct TestDataStructureInner {
        description: String,
        context: ElectionEventContext,
        output: Output,
    }

    #[test]
    #[ignore = "test data are not aligned to the productive data of the verifier"]
    fn test_hash_election_event_context() {
        let json = std::fs::read_to_string(
            test_resources_path()
                .join("get-hash-election-event-context.json")
                .as_path(),
        )
        .unwrap();
        let test_cases: Vec<TestDataStructureInner> = serde_json::from_str(&json).unwrap();
        for tc in test_cases.iter() {
            let hash = HashableMessage::from(&tc.context).recursive_hash();
            assert!(hash.is_ok(), "{}", &hash.unwrap_err());
            assert_eq!(
                hash.unwrap().base64_encode().unwrap(),
                tc.output.d,
                "{}",
                tc.description
            )
        }
    }
}
