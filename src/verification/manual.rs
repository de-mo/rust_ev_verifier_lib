use super::{
    meta_data::VerificationMetaDataList, VerficationsWithErrorAndFailuresType, VerificationError,
    VerificationPeriod, VerificationStatus,
};
use crate::{
    config::VerifierConfig,
    data_structures::context::election_event_configuration::ManuelVerificationInputFromConfiguration,
    file_structure::{
        tally_directory::BBDirectoryTrait, ContextDirectoryTrait, TallyDirectoryTrait,
        VerificationDirectoryTrait,
    },
};
use chrono::NaiveDate;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::EncodeTrait;
use std::collections::HashMap;

/// Trait to get the information of the manual verifications in form of string
///
/// The trait can be used to print the information of the manual verifications
pub trait ManualVerificationInformationTrait {
    /// Get the fingerprints of the direct trust certificates
    ///
    /// Return a [Vec] of a tuples:
    /// - The first element of the tuple is the authority of the certificate
    /// - The second element of the tuple is the fingerprint
    fn dt_fingerprints_to_key_value(&self) -> Vec<(String, String)>;

    /// Get the verification directory path as string
    fn verification_directory_path(&self) -> String;

    /// Get the various information related to the manuel verifications
    ///
    /// Return a [Vec] of a tuples:
    /// - The first element of the tuple is the the name of the information
    /// - The second element of the tuple is the information
    fn information_to_key_value(&self) -> Vec<(String, String)>;

    /// Get the stati of the verification
    ///
    /// Return a [Vec] of a tuples:
    /// - The first element of the tuple is the the id/name of the verification
    /// - The second element of the tuple is the status
    ///
    /// Return empty if it is not relevant
    fn verification_stati_to_key_value(&self) -> Vec<(String, String)> {
        vec![]
    }
}

/// Data for the manual verifications, containing all the data that
/// are necessary for Setup and for Tally
struct ManualVerificationsForAllPeriod<D: VerificationDirectoryTrait + Clone> {
    verification_directory: D,
    direct_trust_certificate_fingerprints: HashMap<String, String>,
    contest_identification: String,
    contest_date: NaiveDate,
    number_of_votes: usize,
    number_of_ballots: usize,
    number_of_elections: usize,
    number_of_productive_voters: usize,
    number_of_test_voters: usize,
    number_of_productive_ballot_boxes: usize,
    number_of_test_ballot_boxes: usize,
}

/// Data for the results of the verifications
///
/// It contains the following information
/// - The metadalist of the verifications to collect some information, like the name
/// - The list of unfinished verifications
/// - The list of verifications with errors or failures (with the the errors and failures as list of strings)
/// - The list of excluded verifications
pub struct VerificationsResult {
    metadata: VerificationMetaDataList,
    verifications_status: HashMap<String, VerificationStatus>,
    verifications_with_errors_and_failures: VerficationsWithErrorAndFailuresType,
    excluded_verifications: Vec<String>,
}

/// Data for the manual verifications on the setup
pub struct ManualVerificationsSetup<D: VerificationDirectoryTrait + Clone> {
    manual_verifications_all_periods: ManualVerificationsForAllPeriod<D>,
    verifications_result: VerificationsResult,
}
/// Data for the manual verifications on the tally
pub struct ManualVerificationsTally<D: VerificationDirectoryTrait + Clone> {
    manual_verifications_all_periods: ManualVerificationsForAllPeriod<D>,
    number_of_test_used_voting_cards: usize,
    number_of_productive_used_voting_cards: usize,
    verifications_result: VerificationsResult,
}

/// Enum for the manual verifications (for setup oder tally)
pub enum ManualVerifications<D: VerificationDirectoryTrait + Clone> {
    Setup(ManualVerificationsSetup<D>),
    Tally(ManualVerificationsTally<D>),
}

impl<D: VerificationDirectoryTrait + Clone> ManualVerificationsForAllPeriod<D> {
    /// Create a new [ManualVerificationsForAllPeriod]
    ///
    /// Inputs
    /// - `directory`: The Verification directory
    /// - `config`: The configuration of the verifier
    pub fn try_new(
        directory: &D,
        config: &'static VerifierConfig,
    ) -> Result<Self, VerificationError> {
        let keystore = config.keystore().map_err(VerificationError::ConfigError)?;
        let fingerprints = keystore
            .fingerprints()
            .map_err(VerificationError::DirectTrust)?
            .iter()
            .map(|(k, v)| (k.to_string(), v.base16_encode().unwrap()))
            .collect::<HashMap<_, _>>();
        let config_dir = directory.context();
        let ee_config = config_dir.election_event_configuration().map_err(|e| {
            VerificationError::FileStructureError {
                msg: "Error reading election_event_configuration".to_string(),
                source: Box::new(e),
            }
        })?;
        let manual_inputs = ManuelVerificationInputFromConfiguration::try_from(ee_config.as_ref())
            .map_err(VerificationError::DataStructure)?;
        Ok(Self {
            verification_directory: directory.clone(),
            direct_trust_certificate_fingerprints: fingerprints,
            contest_identification: manual_inputs.contest_identification,
            contest_date: manual_inputs.contest_date,
            number_of_votes: manual_inputs.number_of_votes,
            number_of_ballots: manual_inputs.number_of_ballots,
            number_of_elections: manual_inputs.number_of_elections,
            number_of_productive_voters: manual_inputs.number_of_productive_voters,
            number_of_test_voters: manual_inputs.number_of_test_voters,
            number_of_productive_ballot_boxes: manual_inputs.number_of_productive_ballot_boxes,
            number_of_test_ballot_boxes: manual_inputs.number_of_test_ballot_boxes,
        })
    }
}

impl<D: VerificationDirectoryTrait + Clone> ManualVerificationInformationTrait
    for ManualVerificationsForAllPeriod<D>
{
    fn dt_fingerprints_to_key_value(&self) -> Vec<(String, String)> {
        let mut res = self
            .direct_trust_certificate_fingerprints
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        res.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        res
    }

    fn information_to_key_value(&self) -> Vec<(String, String)> {
        vec![
            (
                "Contest Identification".to_string(),
                self.contest_identification.to_string(),
            ),
            ("Contest Date".to_string(), self.contest_date.to_string()),
            (
                "Number of votes".to_string(),
                self.number_of_votes.to_string(),
            ),
            (
                "Number of vote objects".to_string(),
                self.number_of_ballots.to_string(),
            ),
            (
                "Number of elections".to_string(),
                self.number_of_elections.to_string(),
            ),
            (
                "Number of voters (productive)".to_string(),
                self.number_of_productive_voters.to_string(),
            ),
            (
                "Number of voters (test)".to_string(),
                self.number_of_test_voters.to_string(),
            ),
            (
                "Number of ballot boxes (productive)".to_string(),
                self.number_of_productive_ballot_boxes.to_string(),
            ),
            (
                "Number of ballot boxes (test)".to_string(),
                self.number_of_test_ballot_boxes.to_string(),
            ),
        ]
    }

    fn verification_directory_path(&self) -> String {
        self.verification_directory.path_to_string()
    }
}

impl VerificationsResult {
    /// Create new [VerificationsResult]
    ///
    /// Inputs:
    /// - `metadata`: The metadalist of the verifications to collect some information, like the name
    /// - `verifications_status`: The verifications (key is verification id) with the status [VerificationStatus]
    /// - `verifications_with_errors_and_failures`: see [VerficationsWithErrorAndFailuresType]
    /// - `excluded_verifications`: The list of excluded verifications. The vector contains the id of the verifications
    ///
    /// It is recommended to deliver in `verifications_with_errors_and_failures` on the verifications having errors or failures. The verification with success should
    /// not be delivered
    pub fn new(
        metadata: &VerificationMetaDataList,
        verifications_status: &HashMap<String, VerificationStatus>,
        verifications_with_errors_and_failures: &VerficationsWithErrorAndFailuresType,
        excluded_verifications: &[String],
    ) -> Self {
        Self {
            metadata: metadata.clone(),
            verifications_status: verifications_status.clone(),
            verifications_with_errors_and_failures: verifications_with_errors_and_failures.clone(),
            excluded_verifications: excluded_verifications.to_vec(),
        }
    }

    /// Get the name of the verification id
    ///
    /// Return empty if the id is not known
    pub fn name_for_verification_id(&self, id: &str) -> String {
        match self.metadata.get(id) {
            Some(v) => v.name().to_string(),
            None => String::default(),
        }
    }

    fn informatiion_to_key_value(&self) -> Vec<(String, String)> {
        let mut res = vec![];
        res.push((
            "Number of verifications".to_string(),
            self.metadata.len().to_string(),
        ));
        res.push((
            "Excluded verifications".to_string(),
            match self.excluded_verifications.is_empty() {
                true => "None".to_string(),
                false => self
                    .excluded_verifications
                    .iter()
                    .map(|id| format!("{}-{}", id, self.name_for_verification_id(id)))
                    .collect::<Vec<_>>()
                    .join(", "),
            },
        ));
        let is_running = self
            .verifications_status
            .iter()
            .any(|(_, v)| *v == VerificationStatus::Running);
        res.push((
            "Run status".to_string(),
            match is_running {
                false => "Finished".to_string(),
                true => "Running".to_string(),
            },
        ));
        if is_running {
            res.push((
                "Number of running verifications".to_string(),
                self.verifications_status
                    .values()
                    .filter(|v| **v == VerificationStatus::Running)
                    .count()
                    .to_string(),
            ));
        }
        res.push((
            "Number of verifications with errors".to_string(),
            format!(
                "{}",
                self.verifications_with_errors_and_failures
                    .values()
                    .filter(|(errors, _)| !errors.is_empty())
                    .count()
            ),
        ));
        res.push((
            "Number of verifications with failures".to_string(),
            format!(
                "{}",
                self.verifications_with_errors_and_failures
                    .values()
                    .filter(|(_, failures)| !failures.is_empty())
                    .count()
            ),
        ));
        res
    }

    fn verification_stati_to_key_value(&self) -> Vec<(String, String)> {
        let mut ids = self.metadata.id_list();
        ids.sort();
        ids.iter()
            .map(|id| {
                let id_string = id.to_string();
                let key = format!("{} ({})", &id_string, self.metadata.get(id).unwrap().name());
                if self.excluded_verifications.contains(&id_string) {
                    return (key, "Excluded".to_string());
                }
                (
                    key,
                    match self.verifications_status.get(&id_string) {
                        Some(v) => v.as_ref().to_string(),
                        None => "Unknown".to_string(),
                    },
                )
            })
            .collect::<Vec<_>>()
    }
}

impl<D: VerificationDirectoryTrait + Clone> ManualVerificationsSetup<D> {
    /// Create new [ManualVerificationsSetup]
    ///
    /// Inputs:
    /// - `directory`: Verification directory
    /// - `config`: Verifier configuration
    /// - `metadata`: The metadalist of the verifications to collect some information, like the name
    /// - `verifications_status`: The verifications (key is verification id) with the status [VerificationStatus]
    /// - `verifications_with_errors_and_failures`: see [VerficationsWithErrorAndFailuresType]
    /// - `excluded_verifications`: The list of excluded verifications. The vector contains the id of the verifications
    ///
    /// It is recommended to deliver in `verifications_with_errors_and_failures` on the verifications having errors or failures. The verification with success should
    /// not be delivered
    pub fn try_new(
        directory: &D,
        config: &'static VerifierConfig,
        metadata: &VerificationMetaDataList,
        verifications_status: &HashMap<String, VerificationStatus>,
        verifications_with_errors_and_failures: &VerficationsWithErrorAndFailuresType,
        excluded_verifications: &[String],
    ) -> Result<Self, VerificationError> {
        Ok(Self {
            manual_verifications_all_periods: ManualVerificationsForAllPeriod::try_new(
                directory, config,
            )?,
            verifications_result: VerificationsResult::new(
                metadata,
                verifications_status,
                verifications_with_errors_and_failures,
                excluded_verifications,
            ),
        })
    }
}

impl<D: VerificationDirectoryTrait + Clone> ManualVerificationInformationTrait
    for ManualVerificationsSetup<D>
{
    fn dt_fingerprints_to_key_value(&self) -> Vec<(String, String)> {
        self.manual_verifications_all_periods
            .dt_fingerprints_to_key_value()
    }

    fn verification_directory_path(&self) -> String {
        self.manual_verifications_all_periods
            .verification_directory_path()
    }

    fn information_to_key_value(&self) -> Vec<(String, String)> {
        let mut res = self
            .manual_verifications_all_periods
            .information_to_key_value();
        res.append(&mut self.verifications_result.informatiion_to_key_value());
        res
    }

    fn verification_stati_to_key_value(&self) -> Vec<(String, String)> {
        self.verifications_result.verification_stati_to_key_value()
    }
}

impl<D: VerificationDirectoryTrait + Clone> ManualVerificationsTally<D> {
    /// Create new [ManualVerificationsTally]
    ///
    /// Inputs:
    /// - `directory`: Verification directory
    /// - `config`: Verifier configuration
    /// - `metadata`: The metadalist of the verifications to collect some information, like the name
    /// - `verifications_status`: The verifications (key is verification id) with the status [VerificationStatus]
    /// - `verifications_with_errors_and_failures`: see [VerficationsWithErrorAndFailures]
    /// - `excluded_verifications`: The list of excluded verifications. The vector contains the id of the verifications
    ///
    /// It is recommended to deliver in `verifications_with_errors_and_failures` on the verifications having errors or failures. The verification with success should
    /// not be delivered
    fn try_new(
        directory: &D,
        config: &'static VerifierConfig,
        metadata: &VerificationMetaDataList,
        verifications_status: &HashMap<String, VerificationStatus>,
        verifications_with_errors_and_failures: &VerficationsWithErrorAndFailuresType,
        excluded_verifications: &[String],
    ) -> Result<Self, VerificationError> {
        let tally_dir = directory.unwrap_tally();
        let config_dir = directory.context();
        let ee_context = config_dir.election_event_context_payload().map_err(|e| {
            VerificationError::FileStructureError {
                msg: "Error reading election_event_context_payload".to_string(),
                source: Box::new(e),
            }
        })?;
        let mut number_of_productive_used_voting_cards = 0;
        let mut number_of_test_used_voting_cards = 0;
        for vcs_context in ee_context
            .election_event_context
            .verification_card_set_contexts
            .iter()
        {
            let bb_id = &vcs_context.ballot_box_id;
            let bb_dir = tally_dir
                .bb_directories()
                .iter()
                .find(|dir| &dir.name() == bb_id)
                .ok_or_else(|| {
                    VerificationError::Generic(format!(
                        "Ballot box Directory {bb_id} not found in tally"
                    ))
                })?;
            let nb_used_vc = bb_dir
                .tally_component_votes_payload()
                .map_err(|e| VerificationError::FileStructureError {
                    msg: format!("Error reading {}/tally_component_votes_payload", bb_id),
                    source: Box::new(e),
                })?
                .votes
                .len();
            match vcs_context.test_ballot_box {
                true => number_of_productive_used_voting_cards += nb_used_vc,
                false => number_of_test_used_voting_cards += nb_used_vc,
            }
        }
        Ok(Self {
            manual_verifications_all_periods: ManualVerificationsForAllPeriod::try_new(
                directory, config,
            )?,
            number_of_productive_used_voting_cards,
            number_of_test_used_voting_cards,
            verifications_result: VerificationsResult::new(
                metadata,
                verifications_status,
                verifications_with_errors_and_failures,
                excluded_verifications,
            ),
        })
    }
}

impl<D: VerificationDirectoryTrait + Clone> ManualVerificationInformationTrait
    for ManualVerificationsTally<D>
{
    fn dt_fingerprints_to_key_value(&self) -> Vec<(String, String)> {
        self.manual_verifications_all_periods
            .dt_fingerprints_to_key_value()
    }

    fn verification_directory_path(&self) -> String {
        self.manual_verifications_all_periods
            .verification_directory_path()
    }

    fn information_to_key_value(&self) -> Vec<(String, String)> {
        let mut res = self
            .manual_verifications_all_periods
            .information_to_key_value();
        res.push((
            "Number of productive voting cards used".to_string(),
            self.number_of_productive_used_voting_cards.to_string(),
        ));
        res.push((
            "Number of test voting cards used".to_string(),
            self.number_of_test_used_voting_cards.to_string(),
        ));
        res.append(&mut self.verifications_result.informatiion_to_key_value());
        res
    }

    fn verification_stati_to_key_value(&self) -> Vec<(String, String)> {
        self.verifications_result.verification_stati_to_key_value()
    }
}

impl<D: VerificationDirectoryTrait + Clone> ManualVerifications<D> {
    /// Create new [ManualVerifications]
    ///
    /// Inputs:
    /// - `period`: Verification period
    /// - `directory`: Verification directory
    /// - `config`: Verifier configuration
    /// - `verifications_status`: The verifications (key is verification id) with the status [VerificationStatus]
    /// - `verifications_with_errors_and_failures`: see [VerficationsWithErrorAndFailuresType]
    /// - `excluded_verifications`: The list of excluded verifications. The vector contains the id of the verifications
    ///
    /// It is recommended to deliver in `verifications_with_errors_and_failures` on the verifications having errors or failures. The verification with success should
    /// not be delivered
    pub fn try_new(
        period: VerificationPeriod,
        directory: &D,
        config: &'static VerifierConfig,
        verifications_status: &HashMap<String, VerificationStatus>,
        verifications_with_errors_and_failures: &VerficationsWithErrorAndFailuresType,
        excluded_verifications: &[String],
    ) -> Result<Self, VerificationError> {
        let meta_data =
            VerificationMetaDataList::load_period(config.get_verification_list_str(), &period)?;
        match period {
            VerificationPeriod::Setup => Ok(ManualVerifications::Setup(
                ManualVerificationsSetup::try_new(
                    directory,
                    config,
                    &meta_data,
                    verifications_status,
                    verifications_with_errors_and_failures,
                    excluded_verifications,
                )?,
            )),
            VerificationPeriod::Tally => Ok(ManualVerifications::Tally(
                ManualVerificationsTally::try_new(
                    directory,
                    config,
                    &meta_data,
                    verifications_status,
                    verifications_with_errors_and_failures,
                    excluded_verifications,
                )?,
            )),
        }
    }
}

impl<D: VerificationDirectoryTrait + Clone> ManualVerificationInformationTrait
    for ManualVerifications<D>
{
    fn dt_fingerprints_to_key_value(&self) -> Vec<(String, String)> {
        match self {
            ManualVerifications::Setup(s) => s.dt_fingerprints_to_key_value(),
            ManualVerifications::Tally(t) => t.dt_fingerprints_to_key_value(),
        }
    }

    fn verification_directory_path(&self) -> String {
        match self {
            ManualVerifications::Setup(s) => s.verification_directory_path(),
            ManualVerifications::Tally(t) => t.verification_directory_path(),
        }
    }

    fn information_to_key_value(&self) -> Vec<(String, String)> {
        match self {
            ManualVerifications::Setup(s) => s.information_to_key_value(),
            ManualVerifications::Tally(t) => t.information_to_key_value(),
        }
    }

    fn verification_stati_to_key_value(&self) -> Vec<(String, String)> {
        match self {
            ManualVerifications::Setup(s) => s.verification_stati_to_key_value(),
            ManualVerifications::Tally(t) => t.verification_stati_to_key_value(),
        }
    }
}
