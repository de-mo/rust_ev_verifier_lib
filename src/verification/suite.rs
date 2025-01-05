//! Module implementing the suite of verifications

use super::{
    meta_data::VerificationMetaDataList, setup::get_verifications as get_verifications_setup,
    tally::get_verifications as get_verifications_tally, verifications::Verification,
    VerificationCategory, VerificationError, VerificationPeriod,
};
use crate::{config::Config, file_structure::VerificationDirectory};

/// Enum for the suite of verifications
pub struct VerificationSuite<'a> {
    period: VerificationPeriod,
    list: VerificationList<'a>,
    exclusion: Vec<String>,
}

/// List of verifications
pub struct VerificationList<'a>(pub Vec<Verification<'a, VerificationDirectory>>);

impl<'a> VerificationSuite<'a> {
    /// Create a new suite
    ///
    /// The function collects all the implemented tests and remove the excluded
    /// verifications. The ids in exclusion that does not exist are ignored
    pub fn new(
        period: &VerificationPeriod,
        metadata_list: &'a VerificationMetaDataList,
        exclusion: &[String],
        config: &'static Config,
    ) -> Result<VerificationSuite<'a>, VerificationError> {
        let all_verifs = match period {
            VerificationPeriod::Setup => get_verifications_setup(metadata_list, config)?,

            VerificationPeriod::Tally => get_verifications_tally(metadata_list, config)?,
        };
        let all_ids: Vec<String> = all_verifs.0.iter().map(|v| v.id().to_string()).collect();
        let verifs = all_verifs
            .0
            .into_iter()
            .filter(|v| !exclusion.contains(&v.id().to_string()))
            .collect::<Vec<_>>();
        let mut excl: Vec<String> = exclusion.iter().map(|s| s.to_string()).collect();
        excl.retain(|s| all_ids.contains(s));
        Ok(VerificationSuite {
            period: *period,
            list: VerificationList(verifs),
            exclusion: excl,
        })
    }

    /// Period of the suite
    pub fn period(&self) -> &VerificationPeriod {
        &self.period
    }

    /// All verifications
    ///
    /// The excluded verifications are not collected
    pub fn verifications(&'a self) -> &'a VerificationList<'a> {
        &self.list
    }

    /// All verifications mutable
    ///
    /// The excluded verifications are not collected
    pub fn verifications_mut(&'a mut self) -> &'a mut VerificationList<'a> {
        &mut self.list
    }

    /// Length of all verifications
    ///
    /// The excluded verifications are not collected
    pub fn len(&self) -> usize {
        self.list.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// List of excluded verifications
    pub fn exclusion(&self) -> &[String] {
        &self.exclusion
    }

    /// Length of excluded verifications
    pub fn len_excluded(&self) -> usize {
        self.exclusion.len()
    }

    /// List of all verifications for a category
    ///
    /// The excluded verifications are not collected
    pub fn get_verifications_for_category(
        &self,
        category: VerificationCategory,
    ) -> Vec<&Verification<'a, VerificationDirectory>> {
        self.list
            .0
            .iter()
            .filter(|e| e.meta_data().category() == &category)
            .collect()
    }

    /// List of ids of all verifications
    ///
    /// The excluded verifications are not collected
    pub fn collect_id(&self) -> Vec<&str> {
        let mut list: Vec<&str> = self.list.0.iter().map(|v| v.id()).collect();
        list.sort();
        list
    }

    /// Find a verification with id
    ///
    /// The excluded verifications are not searchable
    pub fn find_by_id(&self, id: &str) -> Option<&Verification<'a, VerificationDirectory>> {
        self.list.0.iter().find(|&v| v.meta_data().id() == id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{config::test::CONFIG_TEST, verification::meta_data::VerificationMetaData};

    #[test]
    fn test_setup_verifications() {
        let metadata_list =
            VerificationMetaDataList::load(CONFIG_TEST.get_verification_list_str()).unwrap();
        let r_verifs = VerificationSuite::new(
            &VerificationPeriod::Setup,
            &metadata_list,
            &[],
            &CONFIG_TEST,
        );
        if r_verifs.is_err() {
            let err = r_verifs.as_ref().err().unwrap();
            println!("{:?}", err)
        }
        assert!(r_verifs.is_ok());
        let verifs = r_verifs.unwrap();
        let setup: Vec<VerificationMetaData> = metadata_list
            .iter()
            .filter(|v| v.period() == &VerificationPeriod::Setup)
            .cloned()
            .collect();
        assert_eq!(verifs.len(), setup.len());
        let verif_ids = verifs.collect_id();
        let metadata_ids: Vec<&str> = setup.iter().map(|v| v.id()).collect();
        assert_eq!(verif_ids, metadata_ids)
    }

    #[test]
    fn test_tally_verifications() {
        let metadata_list =
            VerificationMetaDataList::load(CONFIG_TEST.get_verification_list_str()).unwrap();
        let r_verifs = VerificationSuite::new(
            &VerificationPeriod::Tally,
            &metadata_list,
            &[],
            &CONFIG_TEST,
        );
        if r_verifs.is_err() {
            let err = r_verifs.as_ref().err().unwrap();
            println!("{:?}", err)
        }
        assert!(r_verifs.is_ok());
        let verifs = r_verifs.unwrap();
        let tally: Vec<VerificationMetaData> = metadata_list
            .iter()
            .filter(|v| v.period() == &VerificationPeriod::Tally)
            .cloned()
            .collect();
        assert_eq!(verifs.len(), tally.len());
        let verif_ids = verifs.collect_id();
        let metadata_ids: Vec<&str> = tally.iter().map(|v| v.id()).collect();
        assert_eq!(verif_ids, metadata_ids)
    }

    #[test]
    fn test_with_exclusion() {
        /*
        let metadata_list =
            VerificationMetaDataList::load(CONFIG_TEST.get_verification_list_str()).unwrap();
        let verifs = VerificationSuite::new(
            &VerificationPeriod::Setup,
            &metadata_list,
            &["02.01".to_string(), "05.01".to_string()],
            &CONFIG_TEST,
        );
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF - 2);
        assert_eq!(verifs.len_excluded(), 2);
        assert_eq!(
            verifs.exclusion,
            vec!["02.01".to_string(), "05.01".to_string()]
        );
        let verifs = VerificationSuite::new(
            &VerificationPeriod::Setup,
            &metadata_list,
            &["toto".to_string()],
            &CONFIG_TEST,
        );
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF);
        assert_eq!(verifs.len_excluded(), 0);
        assert!(verifs.exclusion.is_empty());
        let verifs = VerificationSuite::new(
            &VerificationPeriod::Setup,
            &metadata_list,
            &["02.01".to_string(), "05.01".to_string(), "toto".to_string()],
            &CONFIG_TEST,
        );
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF - 2);
        assert_eq!(verifs.len_excluded(), 2);
        assert_eq!(
            verifs.exclusion,
            vec!["02.01".to_string(), "05.01".to_string()]
        ); */
    }
}
