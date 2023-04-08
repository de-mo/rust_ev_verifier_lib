use crate::file_structure::VerificationDirectory;

use super::error::{VerificationError, VerificationFailure};
use super::{VerificationCategory, VerificationPeriod, VerificationStatus};
use log::{info, warn};
use std::time::{Duration, SystemTime};

pub struct VerificationMetaData {
    pub id: String,
    pub nr: String,
    pub name: String,
    pub period: VerificationPeriod,
    pub category: VerificationCategory,
}

pub struct Verification {
    pub meta_data: VerificationMetaData,
    status: VerificationStatus,
    verification_fn:
        Box<dyn Fn(&VerificationDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>)>,
    duration: Option<Duration>,
    errors: Box<Vec<VerificationError>>,
    failures: Box<Vec<VerificationFailure>>,
}

impl Verification {
    pub fn new(
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
        info!(
            "Verification {} ({}) started",
            self.meta_data.name, self.meta_data.id
        );
        let (errors, failures) = (self.verification_fn)(directory);
        self.duration = Some(start_time.elapsed().unwrap());
        if !errors.is_empty() {
            self.status = VerificationStatus::Error;
            self.errors = Box::new(errors);
            warn!(
                "Verification {} ({}) finished with errors. Duration: {}s",
                self.meta_data.name,
                self.meta_data.id,
                self.duration.unwrap().as_secs_f32()
            );
        } else {
            if !failures.is_empty() {
                self.status = VerificationStatus::Failed;
                self.failures = Box::new(failures);
                warn!(
                    "Verification {} ({}) finished with failures. Duration: {}s",
                    self.meta_data.name,
                    self.meta_data.id,
                    self.duration.unwrap().as_secs_f32()
                );
            } else {
                self.status = VerificationStatus::Success;
                info!(
                    "Verification {} ({}) finished successfully. Duration: {}s",
                    self.meta_data.name,
                    self.meta_data.id,
                    self.duration.unwrap().as_secs_f32()
                );
            }
        }
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
        fn ok(_: &VerificationDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
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
        fn error(_: &VerificationDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
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
            _: &VerificationDirectory,
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
