//! Module implementing the suite of verifications

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
pub type VerificationList<'a> = Vec<Verification<'a>>;

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
    ) -> Vec<&Verification> {
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
    pub fn find_by_id(&self, id: &str) -> Option<&Verification> {
        self.list.iter().find(|&v| v.meta_data.id == id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXPECTED_IMPL_SETUP_VERIF: usize = 18;
    const IMPL_SETUP_TESTS: &[&str] = &[
        "s100", "s200", "s203", "s300", "s301", "s302", "s303", "s304", "s305", "s306", "s307",
        "s308", "s312", "s400", "s500", "s501", "s502", "s503",
    ];
    const MISSING_SETUP_TESTS: &[&str] = &[
        "s201", "s202", "s204", "s205", "s206", "s207", "s309", "s310", "s311", "s313", "s314",
        "s504", "s505",
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
            &Some(vec!["s200".to_string(), "s500".to_string()]),
        );
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF - 2);
        assert_eq!(verifs.len_excluded(), 2);
        assert_eq!(
            verifs.exclusion,
            vec!["s200".to_string(), "s500".to_string()]
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
                "s200".to_string(),
                "s500".to_string(),
                "toto".to_string(),
            ]),
        );
        assert_eq!(verifs.len(), EXPECTED_IMPL_SETUP_VERIF - 2);
        assert_eq!(verifs.len_excluded(), 2);
        assert_eq!(
            verifs.exclusion,
            vec!["s200".to_string(), "s500".to_string()]
        );
    }
}
