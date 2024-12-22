use super::{meta_data::VerificationMetaDataList, VerificationError, VerificationPeriod};
use crate::{
    config::Config,
    data_structures::context::election_event_configuration::ManuelVerificationInputFromConfiguration,
    file_structure::{
        tally_directory::BBDirectoryTrait, ContextDirectoryTrait, TallyDirectoryTrait,
        VerificationDirectoryTrait,
    },
};
use chrono::NaiveDate;
use rust_ev_crypto_primitives::EncodeTrait;
use std::collections::HashMap;

/// Trait to show the information of the manual verifications
pub trait ManualVerificationInformationTrait {
    fn fingerprints(&self) -> Vec<(String, String)>;
    fn verification_directory_path(&self) -> String;
    fn to_title_value(&self) -> Vec<(String, String)>;
    fn to_string(&self) -> String {
        let mut data = vec![
            (String::new(), String::new()),
            (
                "Verification Directory:".to_string(),
                self.verification_directory_path(),
            ),
            (String::new(), String::new()),
            ("Fingerprints".to_string(), String::new()),
        ];
        data.append(&mut self.fingerprints());
        data.append(&mut vec![
            (String::new(), String::new()),
            ("Information".to_string(), String::new()),
        ]);
        data.append(&mut self.to_title_value());
        let max_title_len = data.iter().map(|(t, _)| t.len()).max().unwrap();
        let res = data
            .iter()
            .map(|(t, v)| match t.is_empty() {
                true => String::new(),
                false => {
                    let mut value = " ".repeat(max_title_len - t.len());
                    value.push_str(v.as_str());
                    format!("{t}: {value}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        res
    }
}

/// Data for the manual verifications, containing all the data that
/// are necessary for Setup and for Tally
pub struct ManualVerificationsAllPeriod<'a, D: VerificationDirectoryTrait> {
    verification_directory: &'a D,
    certificate_fingerprints: HashMap<String, String>,
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

/// Data for the manual verifications on the setup
pub struct ManualVerificationsSetup<'a, D: VerificationDirectoryTrait> {
    manual_verifications_all_periods: ManualVerificationsAllPeriod<'a, D>,
    number_of_tests: u8,
}

/// Data for the manual verifications on the tally
pub struct ManualVerificationsTally<'a, D: VerificationDirectoryTrait> {
    manual_verifications_all_periods: ManualVerificationsAllPeriod<'a, D>,
    number_of_test_used_voting_cards: usize,
    number_of_productive_used_voting_cards: usize,
    number_of_tests: u8,
}

/// Data for the manual verifications
pub enum ManualVerifications<'a, D: VerificationDirectoryTrait> {
    Setup(ManualVerificationsSetup<'a, D>),
    Tally(ManualVerificationsTally<'a, D>),
}

impl<'a, D: VerificationDirectoryTrait> ManualVerificationsAllPeriod<'a, D> {
    pub fn new(directory: &'a D, config: &'static Config) -> Result<Self, VerificationError> {
        let keystore = config.keystore().map_err(VerificationError::DirectTrust)?;
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
            verification_directory: directory,
            certificate_fingerprints: fingerprints,
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

    pub fn verification_directory(&self) -> &D {
        self.verification_directory
    }
}

impl<D: VerificationDirectoryTrait> ManualVerificationInformationTrait
    for ManualVerificationsAllPeriod<'_, D>
{
    fn fingerprints(&self) -> Vec<(String, String)> {
        /*vec![
            ("canton".to_string(), self.certificate_fingerprints.get("canton").unwrap()),
            ("sdm_config".to_string(), self.certificate_fingerprints.get("sdm_config").unwrap()),
            ("sdm_tally".to_string(), self.certificate_fingerprints.get("sdm_tally").unwrap()),
            ("canton".to_string(), self.certificate_fingerprints.get("canton").unwrap()),
            ("canton".to_string(), self.certificate_fingerprints.get("canton").unwrap()),
            ("canton".to_string(), self.certificate_fingerprints.get("canton").unwrap()),
            ("canton".to_string(), self.certificate_fingerprints.get("canton").unwrap()),
        ]*/
        let mut res = self
            .certificate_fingerprints
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        res.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        res
    }

    fn to_title_value(&self) -> Vec<(String, String)> {
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

impl<'a, D: VerificationDirectoryTrait> ManualVerificationsSetup<'a, D> {
    pub fn new(
        directory: &'a D,
        config: &'static Config,
        metadata: &VerificationMetaDataList,
    ) -> Result<Self, VerificationError> {
        Ok(Self {
            manual_verifications_all_periods: ManualVerificationsAllPeriod::new(directory, config)?,
            number_of_tests: metadata.len() as u8,
        })
    }
}

impl<D: VerificationDirectoryTrait> ManualVerificationInformationTrait
    for ManualVerificationsSetup<'_, D>
{
    fn fingerprints(&self) -> Vec<(String, String)> {
        self.manual_verifications_all_periods.fingerprints()
    }

    fn verification_directory_path(&self) -> String {
        self.manual_verifications_all_periods
            .verification_directory_path()
    }

    fn to_title_value(&self) -> Vec<(String, String)> {
        let mut res = self.manual_verifications_all_periods.to_title_value();
        res.push((
            "Number of Tests".to_string(),
            self.number_of_tests.to_string(),
        ));
        res
    }
}

impl<'a, D: VerificationDirectoryTrait> ManualVerificationsTally<'a, D> {
    fn new(
        directory: &'a D,
        config: &'static Config,
        metadata: &VerificationMetaDataList,
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
            manual_verifications_all_periods: ManualVerificationsAllPeriod::new(directory, config)?,
            number_of_productive_used_voting_cards,
            number_of_test_used_voting_cards,
            number_of_tests: metadata.len() as u8,
        })
    }
}

impl<D: VerificationDirectoryTrait> ManualVerificationInformationTrait
    for ManualVerificationsTally<'_, D>
{
    fn fingerprints(&self) -> Vec<(String, String)> {
        self.manual_verifications_all_periods.fingerprints()
    }

    fn verification_directory_path(&self) -> String {
        self.manual_verifications_all_periods
            .verification_directory_path()
    }

    fn to_title_value(&self) -> Vec<(String, String)> {
        let mut res = self.manual_verifications_all_periods.to_title_value();
        res.push((
            "Number of productive voting cards used".to_string(),
            self.number_of_productive_used_voting_cards.to_string(),
        ));
        res.push((
            "Number of test voting cards used".to_string(),
            self.number_of_test_used_voting_cards.to_string(),
        ));
        res.push((
            "Number of Tests".to_string(),
            self.number_of_tests.to_string(),
        ));
        res
    }
}

impl<'a, D: VerificationDirectoryTrait> ManualVerifications<'a, D> {
    pub fn new(
        period: VerificationPeriod,
        directory: &'a D,
        config: &'static Config,
    ) -> Result<Self, VerificationError> {
        let meta_data =
            VerificationMetaDataList::load_period(config.get_verification_list_str(), &period)?;
        match period {
            VerificationPeriod::Setup => Ok(ManualVerifications::Setup(
                ManualVerificationsSetup::new(directory, config, &meta_data)?,
            )),
            VerificationPeriod::Tally => Ok(ManualVerifications::Tally(
                ManualVerificationsTally::new(directory, config, &meta_data)?,
            )),
        }
    }
}

impl<D: VerificationDirectoryTrait> ManualVerificationInformationTrait
    for ManualVerifications<'_, D>
{
    fn fingerprints(&self) -> Vec<(String, String)> {
        match self {
            ManualVerifications::Setup(s) => s.fingerprints(),
            ManualVerifications::Tally(t) => t.fingerprints(),
        }
    }

    fn verification_directory_path(&self) -> String {
        match self {
            ManualVerifications::Setup(s) => s.verification_directory_path(),
            ManualVerifications::Tally(t) => t.verification_directory_path(),
        }
    }

    fn to_title_value(&self) -> Vec<(String, String)> {
        match self {
            ManualVerifications::Setup(s) => s.to_title_value(),
            ManualVerifications::Tally(t) => t.to_title_value(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let period = VerificationPeriod::Setup;
        let data = ManualVerifications::new(period, &dir, &CONFIG_TEST).unwrap();
        println!("{}", data.to_string());
    }
}
