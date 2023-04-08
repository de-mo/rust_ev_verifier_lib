pub mod error;
pub mod setup;
pub mod tally;
pub mod verification;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum VerificationCategory {
    Authenticity,
    Consistency,
    Completness,
    Integrity,
    Evidence,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    Stopped,
    Started,
    Success,
    Error,
    Failed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VerificationPeriod {
    Setup,
    Tally,
}

pub type VerificationList = Vec<verification::Verification>;

pub trait VerificationListTrait {
    fn len(&self) -> usize;
    fn len_with_exclusion(&self, exclusion: &Vec<&String>) -> usize;
    fn len_excluded(&self, exclusion: &Vec<&String>) -> usize;
    fn get_verifications(&self, category: VerificationCategory)
        -> Vec<&verification::Verification>;
    fn collect_id(&self) -> Vec<String>;
    fn find_by_id(&self, id: &str) -> Option<&verification::Verification>;
}

impl VerificationListTrait for VerificationList {
    fn len(&self) -> usize {
        self.len()
    }

    fn len_with_exclusion(&self, exclusion: &Vec<&String>) -> usize {
        self.iter()
            .filter(|e| !exclusion.contains(&&e.meta_data.id))
            .count()
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
        let mut list: Vec<String> = self.iter().map(|v| v.meta_data.id.clone()).collect();
        list.sort();
        list
    }

    fn find_by_id(&self, id: &str) -> Option<&verification::Verification> {
        self.iter().find(|&v| v.meta_data.id == id)
    }
}

pub enum VerificationsForPeriod {
    Setup(Box<VerificationList>),
    Tally(Box<VerificationList>),
}

impl VerificationListTrait for VerificationsForPeriod {
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
    pub fn new(period: VerificationPeriod) -> VerificationsForPeriod {
        match period {
            VerificationPeriod::Setup => {
                VerificationsForPeriod::Setup(Box::new(setup::get_verifications()))
            }
            VerificationPeriod::Tally => {
                VerificationsForPeriod::Tally(Box::new(tally::get_verifications()))
            }
        }
    }

    pub fn value(&self) -> &VerificationList {
        match self {
            VerificationsForPeriod::Setup(vs) => vs.as_ref(),
            VerificationsForPeriod::Tally(vs) => vs.as_ref(),
        }
    }

    pub fn value_mut(&mut self) -> &mut VerificationList {
        match self {
            VerificationsForPeriod::Setup(vs) => vs.as_mut(),
            VerificationsForPeriod::Tally(vs) => vs.as_mut(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SETUP_EXPECTED: &[(&str, &str, &str, VerificationCategory)] = &[
        (
            "100",
            "3.1",
            "VerifySetupCompleteness",
            VerificationCategory::Completness,
        ),
        (
            "200",
            "2.01",
            "VerifySignatureSetupComponentEncryptionParameters",
            VerificationCategory::Authenticity,
        ),
        (
            "300",
            "3.01",
            "VerifyEncryptionGroupConsistency",
            VerificationCategory::Consistency,
        ),
        (
            "301",
            "3.02",
            "VerifySetupFileNamesConsistency",
            VerificationCategory::Consistency,
        ),
        (
            "302",
            "3.03",
            "VerifyCCRChoiceReturnCodesPublicKeyConsistency",
            VerificationCategory::Consistency,
        ),
        (
            "303",
            "3.04",
            "VerifyCcmElectionPublicKeyConsistency",
            VerificationCategory::Consistency,
        ),
        (
            "304",
            "3.05",
            "VerifyCcmAndCcrSchnorrProofsConsistency",
            VerificationCategory::Consistency,
        ),
        (
            "305",
            "3.06",
            "VerifyChoiceReturnCodesPublicKeyConsistency",
            VerificationCategory::Consistency,
        ),
        ("400", "3.4", "Integrity", VerificationCategory::Integrity),
        (
            "500",
            "5.01",
            "VerifyEncryptionParameters",
            VerificationCategory::Evidence,
        ),
        (
            "501",
            "5.02",
            "VerifySmallPrimeGroupMembers",
            VerificationCategory::Evidence,
        ),
    ];

    #[test]
    fn test_setup_verifications() {
        let verifs = VerificationsForPeriod::new(VerificationPeriod::Setup);
        assert_eq!(verifs.len(), SETUP_EXPECTED.len());
        let mut verifs_id: Vec<&str> = SETUP_EXPECTED.iter().map(|e| e.0).collect();
        verifs_id.sort();
        assert_eq!(verifs.collect_id(), verifs_id)
    }

    #[test]
    fn test_setup_metadata() {
        let verifs = VerificationsForPeriod::new(VerificationPeriod::Setup);
        for (id, nr, name, cat) in SETUP_EXPECTED.iter() {
            let v_md = verifs.find_by_id(id).unwrap();
            assert_eq!(&v_md.meta_data.id, id, "id: {}", id);
            assert_eq!(&v_md.meta_data.nr, nr, "id: {}", id);
            assert_eq!(&v_md.meta_data.name, name, "id: {}", id);
            assert_eq!(&v_md.meta_data.category, cat, "id: {}", id);
        }
    }

    #[test]
    fn test_len_with_exclusion() {
        let verifs = VerificationsForPeriod::new(VerificationPeriod::Setup);
        assert_eq!(
            verifs.len_with_exclusion(&vec![&"200".to_string(), &"500".to_string()]),
            SETUP_EXPECTED.len() - 2
        );
        assert_eq!(
            verifs.len_excluded(&vec![&"200".to_string(), &"500".to_string()]),
            2
        );
        assert_eq!(
            verifs.len_with_exclusion(&vec![&"toto".to_string()]),
            SETUP_EXPECTED.len()
        );
        assert_eq!(verifs.len_excluded(&vec![&"toto".to_string()]), 0);
        assert_eq!(
            verifs.len_with_exclusion(&vec![
                &"200".to_string(),
                &"500".to_string(),
                &"toto".to_string()
            ]),
            SETUP_EXPECTED.len() - 2
        );
        assert_eq!(
            verifs.len_excluded(&vec![
                &"200".to_string(),
                &"500".to_string(),
                &"toto".to_string()
            ]),
            2
        );
    }
}
