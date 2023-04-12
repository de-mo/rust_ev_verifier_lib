//! Module implementing all the verifications

use std::fmt::Display;

use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

use self::meta_data::{VerificationMetaDataList, VerificationMetaDataListTrait};

pub mod error;
pub mod meta_data;
pub mod setup;
pub mod tally;
pub mod verification;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VerificationCategory {
    Authenticity,
    Consistency,
    Completness,
    Integrity,
    Evidence,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    Stopped,
    Running,
    Finished,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VerificationPeriod {
    Setup,
    Tally,
}

pub type VerificationSuite = Vec<verification::Verification>;

pub trait VerificationSuiteTrait {
    fn len(&self) -> usize;
    fn len_with_exclusion(&self, exclusion: &Vec<&String>) -> usize;
    fn len_excluded(&self, exclusion: &Vec<&String>) -> usize;
    fn get_verifications(&self, category: VerificationCategory)
        -> Vec<&verification::Verification>;
    fn collect_id(&self) -> Vec<String>;
    fn find_by_id(&self, id: &str) -> Option<&verification::Verification>;
}

impl VerificationSuiteTrait for VerificationSuite {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_with_exclusion(&self, exclusion: &Vec<&String>) -> usize {
        self.iter().filter(|e| !exclusion.contains(&&e.id)).count()
    }

    fn len_excluded(&self, exclusion: &Vec<&String>) -> usize {
        self.len() - self.len_with_exclusion(exclusion)
    }

    fn get_verifications(
        &self,
        category: VerificationCategory,
    ) -> Vec<&verification::Verification> {
        self.iter()
            .filter(|e| e.meta_data.category == category)
            .collect()
    }

    fn collect_id(&self) -> Vec<String> {
        let mut list: Vec<String> = self.iter().map(|v| v.id.clone()).collect();
        list.sort();
        list
    }

    fn find_by_id(&self, id: &str) -> Option<&verification::Verification> {
        self.iter().find(|&v| v.meta_data.id == id)
    }
}

pub enum VerificationsForPeriod {
    Setup(Box<VerificationSuite>),
    Tally(Box<VerificationSuite>),
}

impl TryFrom<&str> for VerificationPeriod {
    type Error = VerificationPreparationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "setup" => Ok(VerificationPeriod::Setup),
            "tally" => Ok(VerificationPeriod::Tally),
            _ => create_result_with_error!(
                VerificationPreparationErrorType::VerificationPeriod,
                format!("Cannot read period from '{}'", value)
            ),
        }
    }
}

impl TryFrom<&String> for VerificationPeriod {
    type Error = VerificationPreparationError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for VerificationCategory {
    type Error = VerificationPreparationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "authenticity" => Ok(VerificationCategory::Authenticity),
            "completness" => Ok(VerificationCategory::Completness),
            "consistency" => Ok(VerificationCategory::Consistency),
            "integrity" => Ok(VerificationCategory::Integrity),
            "evidence" => Ok(VerificationCategory::Evidence),
            _ => create_result_with_error!(
                VerificationPreparationErrorType::VerificationPeriod,
                format!("Cannot read category from '{}'", value)
            ),
        }
    }
}

impl TryFrom<&String> for VerificationCategory {
    type Error = VerificationPreparationError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl VerificationSuiteTrait for VerificationsForPeriod {
    fn len(&self) -> usize {
        match self {
            VerificationsForPeriod::Setup(b) => b.len(),
            VerificationsForPeriod::Tally(b) => b.len(),
        }
    }

    fn len_with_exclusion(&self, exclusion: &Vec<&String>) -> usize {
        match self {
            VerificationsForPeriod::Setup(b) => b.len_with_exclusion(exclusion),
            VerificationsForPeriod::Tally(b) => b.len_with_exclusion(exclusion),
        }
    }

    fn len_excluded(&self, exclusion: &Vec<&String>) -> usize {
        match self {
            VerificationsForPeriod::Setup(b) => b.len_excluded(exclusion),
            VerificationsForPeriod::Tally(b) => b.len_excluded(exclusion),
        }
    }

    fn get_verifications(
        &self,
        category: VerificationCategory,
    ) -> Vec<&verification::Verification> {
        match self {
            VerificationsForPeriod::Setup(b) => b.get_verifications(category),
            VerificationsForPeriod::Tally(b) => b.get_verifications(category),
        }
    }

    fn collect_id(&self) -> Vec<String> {
        match self {
            VerificationsForPeriod::Setup(b) => b.collect_id(),
            VerificationsForPeriod::Tally(b) => b.collect_id(),
        }
    }

    fn find_by_id(&self, id: &str) -> Option<&verification::Verification> {
        match self {
            VerificationsForPeriod::Setup(b) => b.find_by_id(id),
            VerificationsForPeriod::Tally(b) => b.find_by_id(id),
        }
    }
}

impl VerificationsForPeriod {
    pub fn new(
        period: VerificationPeriod,
        metadata_list: &VerificationMetaDataList,
    ) -> VerificationsForPeriod {
        match period {
            VerificationPeriod::Setup => {
                VerificationsForPeriod::Setup(Box::new(setup::get_verifications(metadata_list)))
            }
            VerificationPeriod::Tally => {
                VerificationsForPeriod::Tally(Box::new(tally::get_verifications(metadata_list)))
            }
        }
    }

    pub fn period(&self) -> VerificationPeriod {
        match self {
            VerificationsForPeriod::Setup(_) => VerificationPeriod::Setup,
            VerificationsForPeriod::Tally(_) => VerificationPeriod::Tally,
        }
    }

    pub fn value(&self) -> &VerificationSuite {
        match self {
            VerificationsForPeriod::Setup(vs) => vs.as_ref(),
            VerificationsForPeriod::Tally(vs) => vs.as_ref(),
        }
    }

    pub fn value_mut(&mut self) -> &mut VerificationSuite {
        match self {
            VerificationsForPeriod::Setup(vs) => vs.as_mut(),
            VerificationsForPeriod::Tally(vs) => vs.as_mut(),
        }
    }

    pub fn get_missing_verifications_id(&self) -> Vec<String> {
        let metadata_list = VerificationMetaDataList::load().unwrap();
        let all_id = metadata_list.id_list_for_period(self.period());
        let verifs_id = self.collect_id();
        let mut diff: Vec<String> = all_id
            .iter()
            .filter(|&x| !verifs_id.contains(x))
            .cloned()
            .collect();
        diff.sort();
        diff
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationPreparationErrorType {
    Metadata,
    VerificationPeriod,
    VerificationCategory,
}

impl Display for VerificationPreparationErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VerificationPreparationErrorType::Metadata => "Meta data",
            VerificationPreparationErrorType::VerificationPeriod => "Verification period",
            VerificationPreparationErrorType::VerificationCategory => "Verification category",
        };
        write!(f, "{s}")
    }
}

pub type VerificationPreparationError = VerifierError<VerificationPreparationErrorType>;

#[cfg(test)]
mod test {
    use crate::verification::meta_data::VerificationMetaDataListTrait;

    use super::*;

    const EXPECTED_IMPL_SETUP_VERIF: usize = 17;
    const IMPL_SETUP_TESTS: &[&str] = &[
        "s100", "s200", "s300", "s301", "s302", "s303", "s304", "s305", "s306", "s307", "s308",
        "s312", "s400", "s500", "s501", "s502", "s503",
    ];
    const MISSING_SETUP_TESTS: &[&str] = &[
        "s201", "s202", "s203", "s204", "s205", "s206", "s207", "s309", "s310", "s311", "s313",
        "s314", "s504", "s505",
    ];

    const EXPECTED_IMPL_TALLY_VERIF: usize = 0;
    const IMPL_TALLY_TESTS: &[&str] = &[];
    const MISSING_TALLY_TESTS: &[&str] = &[
        "t100", "t200", "t201", "t202", "t203", "t204", "t205", "t206", "t300", "t301", "t302",
        "t303", "t304", "t305", "t306", "t307", "t308", "t309", "t310", "t400", "t500", "t501",
    ];

    #[test]
    fn test_setup_verifications() {
        let metadata_list = VerificationMetaDataList::load().unwrap();
        let verifs = VerificationsForPeriod::new(VerificationPeriod::Setup, &metadata_list);
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF);
        assert_eq!(verifs.collect_id(), IMPL_SETUP_TESTS);
        assert_eq!(verifs.get_missing_verifications_id(), MISSING_SETUP_TESTS);
    }

    #[test]
    fn test_tally_verifications() {
        let metadata_list = VerificationMetaDataList::load().unwrap();
        let verifs = VerificationsForPeriod::new(VerificationPeriod::Tally, &metadata_list);
        assert_eq!(verifs.len(), EXPECTED_IMPL_TALLY_VERIF);
        assert_eq!(verifs.collect_id(), IMPL_TALLY_TESTS);
        assert_eq!(verifs.get_missing_verifications_id(), MISSING_TALLY_TESTS);
    }

    #[test]
    fn test_len_with_exclusion() {
        let metadata_list = VerificationMetaDataList::load().unwrap();
        let verifs = VerificationsForPeriod::new(VerificationPeriod::Setup, &metadata_list);
        assert_eq!(
            verifs.len_with_exclusion(&vec![&"s200".to_string(), &"s500".to_string()]),
            EXPECTED_IMPL_SETUP_VERIF - 2
        );
        assert_eq!(
            verifs.len_excluded(&vec![&"s200".to_string(), &"s500".to_string()]),
            2
        );
        assert_eq!(
            verifs.len_with_exclusion(&vec![&"toto".to_string()]),
            EXPECTED_IMPL_SETUP_VERIF
        );
        assert_eq!(verifs.len_excluded(&vec![&"toto".to_string()]), 0);
        assert_eq!(
            verifs.len_with_exclusion(&vec![
                &"s200".to_string(),
                &"s500".to_string(),
                &"toto".to_string()
            ]),
            EXPECTED_IMPL_SETUP_VERIF - 2
        );
        assert_eq!(
            verifs.len_excluded(&vec![
                &"s200".to_string(),
                &"s500".to_string(),
                &"toto".to_string()
            ]),
            2
        );
    }
}
