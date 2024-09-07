use anyhow::Context;
use chrono::NaiveDate;
use rust_ev_crypto_primitives::Encode;
use std::collections::HashMap;

use crate::{
    config::Config, direct_trust::CertificateAuthority, file_structure::VerificationDirectoryTrait,
};

use super::{meta_data::VerificationMetaDataList, VerificationPeriod};

/// Data for the manual verifications, containing all the data that
/// are necessary for Setup and for Tally
struct ManualVerificationsAllPeriod<'a, D: VerificationDirectoryTrait> {
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
struct ManualVerificationsSetup<'a, D: VerificationDirectoryTrait> {
    manual_verifications_all_periods: ManualVerificationsAllPeriod<'a, D>,
    number_of_tests: u8,
}

/// Data for the manual verifications on the tally
struct ManualVerificationsTally<'a, D: VerificationDirectoryTrait> {
    manual_verifications_all_periods: ManualVerificationsAllPeriod<'a, D>,
    number_of_tests: u8,
}

/// Data for the manual verifications
enum ManualVerifications<'a, D: VerificationDirectoryTrait> {
    Setup(ManualVerificationsSetup<'a, D>),
    Tally(ManualVerificationsTally<'a, D>),
}

impl<'a, D: VerificationDirectoryTrait> ManualVerificationsAllPeriod<'a, D> {
    pub fn new(directory: &'a D, config: &'static Config) -> anyhow::Result<Self> {
        let keystore = config.keystore()?;
        let mut fingerprints = HashMap::new();
        for ca in CertificateAuthority::iter() {
            fingerprints.insert(
                ca.to_string(),
                keystore
                    .0
                    .public_certificate(&ca.to_string())?
                    .signing_certificate()
                    .digest()?
                    .base16_encode()
                    .unwrap(),
            );
        }
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
    ) -> anyhow::Result<Self> {
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
    ) -> anyhow::Result<Self> {
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
    ) -> anyhow::Result<Self> {
        let meta_data =
            VerificationMetaDataList::load_period(config.get_verification_list_str(), &period)?;
        match period {
            VerificationPeriod::Setup => Ok(ManualVerifications::Setup(
                ManualVerificationsSetup::new(directory, config, &meta_data)
                    .context("Error creating manual verifications for setup")?,
            )),
            VerificationPeriod::Tally => Ok(ManualVerifications::Tally(
                ManualVerificationsTally::new(directory, config, &meta_data)
                    .context("Error creating manual verifications for tally")?,
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
