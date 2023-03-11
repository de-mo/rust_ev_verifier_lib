use crate::file_structure::directory::VerificationDirectory;

use super::error::{VerificationError, VerificationFailure};
use std::time::{Duration, SystemTime};

use super::{VerificationCategory, VerificationPeriod, VerificationStatus};

pub struct VerificationMetaData {
    id: String,
    nr: String,
    name: String,
    period: VerificationPeriod,
    category: VerificationCategory,
}

pub struct Verification {
    meta_data: VerificationMetaData,
    status: VerificationStatus,
    verification_fn:
        Box<dyn Fn(&VerificationDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>)>,
    duration: Option<Duration>,
    errors: Box<Vec<VerificationError>>,
    failures: Box<Vec<VerificationFailure>>,
}

impl Verification {
    fn new(
        meta_data: VerificationMetaData,
        verification_fn: impl Fn(&VerificationDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>)
            + 'static,
    ) -> Self {
        Verification {
            meta_data,
            status: VerificationStatus::Stopped,
            verification_fn: Box::new(verification_fn),
            duration: None,
            errors: Box::new(vec![]),
            failures: Box::new(vec![]),
        }
    }

    pub fn run(&mut self, directory: &VerificationDirectory) {
        self.status = VerificationStatus::Started;
        let start_time = SystemTime::now();
        let (errors, failures) = (self.verification_fn)(directory);
        if !errors.is_empty() {
            self.status = VerificationStatus::Error;
            self.errors = Box::new(errors);
        } else {
            if !failures.is_empty() {
                self.status = VerificationStatus::Failed;
                self.failures = Box::new(failures);
            } else {
                self.status = VerificationStatus::Success;
            }
        }
        self.duration = Some(start_time.elapsed().unwrap());
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::error::{create_verifier_error, VerifierError};
    use crate::verification::error::{VerificationErrorType, VerificationFailureType};

    use super::*;

    #[test]
    fn run_ok() {
        fn ok(d: &VerificationDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
            (vec![], vec![])
        }
        let mut verif = Verification::new(
            VerificationMetaData {
                id: "test_ok".to_string(),
                nr: "1".to_string(),
                name: "test_ok".to_string(),
                period: VerificationPeriod::Setup,
                category: VerificationCategory::Authenticity,
            },
            Box::new(ok),
        );
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(verif.errors.is_empty());
        assert!(verif.failures.is_empty());
        verif.run(&VerificationDirectory::new(
            VerificationPeriod::Setup,
            &Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Success);
        assert!(verif.errors.is_empty());
        assert!(verif.failures.is_empty());
    }

    #[test]
    fn run_error() {
        fn error(d: &VerificationDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
            (
                vec![
                    create_verifier_error!(VerificationErrorType::Error, "toto"),
                    create_verifier_error!(VerificationErrorType::Error, "toto"),
                ],
                vec![create_verifier_error!(
                    VerificationFailureType::Failure,
                    "toto"
                )],
            )
        }
        let mut verif = Verification::new(
            VerificationMetaData {
                id: "test_ok".to_string(),
                nr: "1".to_string(),
                name: "test_ok".to_string(),
                period: VerificationPeriod::Setup,
                category: VerificationCategory::Authenticity,
            },
            Box::new(error),
        );
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(verif.errors.is_empty());
        assert!(verif.failures.is_empty());
        verif.run(&VerificationDirectory::new(
            VerificationPeriod::Setup,
            &Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Error);
        assert_eq!(verif.errors.len(), 2);
        assert!(verif.failures.is_empty());
    }

    #[test]
    fn run_failure() {
        fn failure(
            d: &VerificationDirectory,
        ) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
            (
                vec![],
                vec![
                    create_verifier_error!(VerificationFailureType::Failure, "toto"),
                    create_verifier_error!(VerificationFailureType::Failure, "toto"),
                ],
            )
        }
        let mut verif = Verification::new(
            VerificationMetaData {
                id: "test_ok".to_string(),
                nr: "1".to_string(),
                name: "test_ok".to_string(),
                period: VerificationPeriod::Setup,
                category: VerificationCategory::Authenticity,
            },
            Box::new(failure),
        );
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(verif.errors.is_empty());
        assert!(verif.failures.is_empty());
        verif.run(&VerificationDirectory::new(
            VerificationPeriod::Setup,
            &Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Failed);
        assert!(verif.errors.is_empty());
        assert_eq!(verif.failures.len(), 2);
    }
}
