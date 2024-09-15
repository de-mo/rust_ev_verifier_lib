use chrono::NaiveDate;
use rust_ev_crypto_primitives::EncodeTrait;
use std::collections::HashMap;

use crate::{config::Config, file_structure::VerificationDirectoryTrait};

use super::{meta_data::VerificationMetaDataList, VerificationError, VerificationPeriod};

/// Data for the manual verifications, containing all the data that
/// are necessary for Setup and for Tally
pub struct ManualVerificationsAllPeriod<'a, D: VerificationDirectoryTrait> {
    verification_directory: &'a D,
    certificate_fingerprints: HashMap<String, String>,
    election_name: u32,
    election_date: NaiveDate,
    number_of_votes: u32,
    number_of_elections: u32,
    number_of_productive_voters: u32,
    number_of_test_voters: u32,
    number_of_productive_ballot_boxes: u32,
    number_of_test_ballot_boxes: u32,
}

/// Data for the manual verifications on the setup
pub struct ManualVerificationsSetup<'a, D: VerificationDirectoryTrait> {
    manual_verifications_all_periods: ManualVerificationsAllPeriod<'a, D>,
    number_of_tests: u8,
}

/// Data for the manual verifications on the tally
pub struct ManualVerificationsTally<'a, D: VerificationDirectoryTrait> {
    manual_verifications_all_periods: ManualVerificationsAllPeriod<'a, D>,
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
        Ok(Self {
            verification_directory: directory,
            certificate_fingerprints: fingerprints,
            election_name: 9999,
            election_date: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
            number_of_votes: 9999,
            number_of_elections: 9999,
            number_of_productive_voters: 9999,
            number_of_test_voters: 9999,
            number_of_productive_ballot_boxes: 9999,
            number_of_test_ballot_boxes: 9999,
        })
    }

    pub fn fingerprints(&self) -> &HashMap<String, String> {
        &self.certificate_fingerprints
    }

    pub fn verification_directory(&self) -> &D {
        self.verification_directory
    }

    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut res = HashMap::new();
        res.insert("Election Name".to_string(), self.election_name.to_string());
        res.insert("Election Date".to_string(), self.election_date.to_string());
        res.insert(
            "Number of votes".to_string(),
            self.number_of_votes.to_string(),
        );
        res.insert(
            "Number of elections".to_string(),
            self.number_of_elections.to_string(),
        );
        res.insert(
            "Number of voters (productive)".to_string(),
            self.number_of_productive_voters.to_string(),
        );
        res.insert(
            "Number of voters (test)".to_string(),
            self.number_of_test_voters.to_string(),
        );
        res.insert(
            "Number of ballot boxes (productive)".to_string(),
            self.number_of_productive_ballot_boxes.to_string(),
        );
        res.insert(
            "Number of ballot boxes (test)".to_string(),
            self.number_of_test_ballot_boxes.to_string(),
        );
        res
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

    pub fn fingerprints(&self) -> &HashMap<String, String> {
        self.manual_verifications_all_periods.fingerprints()
    }

    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut res = self.manual_verifications_all_periods.to_hashmap();
        res.insert(
            "Number of Tests".to_string(),
            self.number_of_tests.to_string(),
        );
        res
    }
}

impl<'a, D: VerificationDirectoryTrait> ManualVerificationsTally<'a, D> {
    fn new(
        directory: &'a D,
        config: &'static Config,
        metadata: &VerificationMetaDataList,
    ) -> Result<Self, VerificationError> {
        Ok(Self {
            manual_verifications_all_periods: ManualVerificationsAllPeriod::new(directory, config)?,
            number_of_tests: metadata.len() as u8,
        })
    }

    pub fn fingerprints(&self) -> &HashMap<String, String> {
        self.manual_verifications_all_periods.fingerprints()
    }

    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut res = self.manual_verifications_all_periods.to_hashmap();
        res.insert(
            "Number of Tests".to_string(),
            self.number_of_tests.to_string(),
        );
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

    pub fn fingerprints(&self) -> &HashMap<String, String> {
        match self {
            ManualVerifications::Setup(s) => s.fingerprints(),
            ManualVerifications::Tally(t) => t.fingerprints(),
        }
    }

    pub fn to_hashmap(&self) -> HashMap<String, String> {
        match self {
            ManualVerifications::Setup(s) => s.to_hashmap(),
            ManualVerifications::Tally(t) => t.to_hashmap(),
        }
    }
}
