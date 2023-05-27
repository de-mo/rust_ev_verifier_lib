//! Module implementing the suite of verifications

use crate::file_structure::VerificationDirectory;

use super::{
    meta_data::{VerificationMetaDataList, VerificationMetaDataListTrait},
    setup::get_verifications as get_verifications_setup,
    tally::get_verifications as get_verifications_tally,
    verification::Verification,
    VerificationCategory, VerificationPeriod,
};

/// Enum for the suite of verifications
pub struct VerificationSuite<'a> {
    period: VerificationPeriod,
    pub list: Box<VerificationList<'a>>,
    exclusion: Vec<String>,
}

/// List of verifications
pub type VerificationList<'a> = Vec<Verification<'a, VerificationDirectory>>;

impl<'a> VerificationSuite<'a> {
    /// Create a new suite
    ///
    /// The function collects all the implemented tests and remove the excluded
    /// verifications. The ids in exclusion that does not exist are ignored
    pub fn new(
        period: &VerificationPeriod,
        metadata_list: &'a VerificationMetaDataList,
        exclusion: &Option<Vec<String>>,
    ) -> VerificationSuite<'a> {
        let mut all_verifs = match period {
            VerificationPeriod::Setup => get_verifications_setup(metadata_list),

            VerificationPeriod::Tally => get_verifications_tally(metadata_list),
        };
        let all_ids: Vec<String> = all_verifs.iter().map(|v| v.id.clone()).collect();
        let mut excl = match exclusion {
            Some(e) => {
                all_verifs.retain(|x| !exclusion.as_ref().unwrap().contains(&&x.id));
                e.iter().map(|f| f.clone()).collect()
            }
            None => vec![],
        };
        excl.retain(|s| all_ids.contains(s));
        if exclusion.is_some() {}
        VerificationSuite {
            period: period.clone(),
            list: Box::new(all_verifs),
            exclusion: excl,
        }
    }

    /// Period of the suite
    pub fn period(&self) -> &VerificationPeriod {
        &self.period
    }

    /// All verifications
    ///
    /// The excluded verifications are not collected
    pub fn verifications(&'a self) -> &'a VerificationList {
        &self.list
    }

    /// All verifications mutable
    ///
    /// The excluded verifications are not collected
    pub fn verifications_mut(&'a mut self) -> &'a mut VerificationList {
        &mut self.list
    }

    /// Get the list of the verifications that are not implemented yet
    pub fn get_not_implemented_verifications_id(
        &self,
        metadata_list: &VerificationMetaDataList,
    ) -> Vec<String> {
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

    /// Length of all verifications
    ///
    /// The excluded verifications are not collected
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// List of excluded verifications
    pub fn exclusion(&self) -> &Vec<String> {
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
            .iter()
            .filter(|e| e.meta_data.category == category)
            .collect()
    }

    /// List of ids of all verifications
    ///
    /// The excluded verifications are not collected
    pub fn collect_id(&self) -> Vec<String> {
        let mut list: Vec<String> = self.list.iter().map(|v| v.id.clone()).collect();
        list.sort();
        list
    }

    /// Find a verification with id
    ///
    /// The excluded verifications are not searchable
    pub fn find_by_id(&self, id: &str) -> Option<&Verification<'a, VerificationDirectory>> {
        self.list.iter().find(|&v| v.meta_data.id == id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXPECTED_IMPL_SETUP_VERIF: usize = 24;
    const IMPL_SETUP_TESTS: &[&str] = &[
        "01.01", "02.01", "02.03", "02.04", "02.05", "02.06", "02.07", "03.01", "03.02", "03.03",
        "03.04", "03.05", "03.06", "03.07", "03.08", "03.09", "03.13", "03.15", "04.01", "05.01",
        "05.02", "05.03", "05.04", "05.21",
    ];
    const MISSING_SETUP_TESTS: &[&str] = &[
        "02.02", "02.08", "03.10", "03.11", "03.12", "03.14", "05.22",
    ];

    const EXPECTED_IMPL_TALLY_VERIF: usize = 2;
    const IMPL_TALLY_TESTS: &[&str] = &["06.01", "09.01"];
    const MISSING_TALLY_TESTS: &[&str] = &[
        "07.01", "07.02", "07.03", "07.04", "07.05", "07.06", "07.07", "08.01", "08.02", "08.03",
        "08.04", "08.05", "08.06", "08.07", "08.08", "08.09", "08.10", "08.11", "10.01", "10.02",
    ];

    #[test]
    fn test_setup_verifications() {
        let metadata_list = VerificationMetaDataList::load().unwrap();
        let verifs = VerificationSuite::new(&VerificationPeriod::Setup, &metadata_list, &None);
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF);
        assert_eq!(verifs.collect_id(), IMPL_SETUP_TESTS);
        assert_eq!(
            verifs.get_not_implemented_verifications_id(&metadata_list),
            MISSING_SETUP_TESTS
        );
    }

    #[test]
    fn test_tally_verifications() {
        let metadata_list = VerificationMetaDataList::load().unwrap();
        let verifs = VerificationSuite::new(&VerificationPeriod::Tally, &metadata_list, &None);
        assert_eq!(verifs.len(), EXPECTED_IMPL_TALLY_VERIF);
        assert_eq!(verifs.collect_id(), IMPL_TALLY_TESTS);
        assert_eq!(
            verifs.get_not_implemented_verifications_id(&metadata_list),
            MISSING_TALLY_TESTS
        );
    }

    #[test]
    fn test_with_exclusion() {
        let metadata_list = VerificationMetaDataList::load().unwrap();
        let verifs = VerificationSuite::new(
            &VerificationPeriod::Setup,
            &metadata_list,
            &Some(vec!["02.01".to_string(), "05.01".to_string()]),
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
            &Some(vec!["toto".to_string()]),
        );
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF);
        assert_eq!(verifs.len_excluded(), 0);
        assert!(verifs.exclusion.is_empty());
        let verifs = VerificationSuite::new(
            &VerificationPeriod::Setup,
            &metadata_list,
            &Some(vec![
                "02.01".to_string(),
                "05.01".to_string(),
                "toto".to_string(),
            ]),
        );
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF - 2);
        assert_eq!(verifs.len_excluded(), 2);
        assert_eq!(
            verifs.exclusion,
            vec!["02.01".to_string(), "05.01".to_string()]
        );
    }
}
