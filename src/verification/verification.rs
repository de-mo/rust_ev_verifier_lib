///! Module implementing the structure of a verification
use super::{
    error::{VerificationError, VerificationFailure},
    meta_data::{VerificationMetaData, VerificationMetaDataList, VerificationMetaDataListTrait},
    VerificationPreparationError, VerificationPreparationErrorType, VerificationStatus,
};
use crate::{
    error::{create_result_with_error, create_verifier_error, VerifierError},
    file_structure::{VerificationDirectory, VerificationDirectoryTrait},
};
use log::{info, warn};
use std::time::{Duration, SystemTime};

/// Struct representing a verification
pub struct Verification<'a, D: VerificationDirectoryTrait> {
    /// Id of the verification
    pub id: String,
    /// Metadata of the verification
    ///
    /// The meta data is a reference to the metadata list loaded from json
    pub meta_data: &'a VerificationMetaData,
    status: VerificationStatus,
    verification_fn: Box<dyn Fn(&D, &mut VerificationResult)>,
    duration: Option<Duration>,
    result: Box<VerificationResult>,
}

/// Struct representing a result of the verification
/// The verification can have many errors and/or many failures
pub struct VerificationResult {
    errors: Vec<VerificationError>,
    failures: Vec<VerificationFailure>,
}

/// Trait defining functions to access the verficiation result
pub trait VerificationResultTrait {
    /// Is the verification ok ?
    ///
    /// If not run, the output is None
    fn is_ok(&self) -> Option<bool>;

    /// Has the verification errors ?
    ///
    /// If not run, the output is None
    fn has_errors(&self) -> Option<bool>;

    /// Has the verification failures ?
    ///
    /// If not run, the output is None
    fn has_failures(&self) -> Option<bool>;

    /// All the errors
    fn errors(&self) -> &Vec<VerificationError>;

    /// All the failures
    fn failures(&self) -> &Vec<VerificationFailure>;
}

impl<'a> Verification<'a, VerificationDirectory> {
    /// Create a new verification.
    ///
    /// The input are the metadata and the explicit function of the verification. The function
    /// must have the following form:
    /// The verification functions should only defined with the traits as follow
    /// ```rust
    /// fn fn_verification<D: VerificationDirectoryTrait>(
    ///    dir: &D,
    ///    result: &mut VerificationResult,
    /// ) {
    ///     ...
    /// }
    /// ```
    ///
    /// The directory contains the directory where the folder setup and tally are located. The result [VerificationResult]
    /// has to be changed according to the results of the verification (pushing errors and/or failures).
    /// The function is called by the method rust of the Verification.
    ///
    /// All the helpers functions called from `fn_verification` have also to take then traits as parameter
    /// and not the structs. Then it is possible to mock the data
    pub fn new(
        id: &str,
        verification_fn: impl Fn(&VerificationDirectory, &mut VerificationResult) + 'static,
        metadata_list: &'a VerificationMetaDataList,
    ) -> Result<Self, VerificationPreparationError> {
        let meta_data = match metadata_list.meta_data_from_id(id) {
            Some(m) => m,
            None => {
                return create_result_with_error!(
                    VerificationPreparationErrorType::Metadata,
                    format!("metadata for id {} not found", id)
                )
            }
        };
        Ok(Verification {
            id: id.to_string(),
            meta_data,
            status: VerificationStatus::Stopped,
            verification_fn: Box::new(verification_fn),
            duration: None,
            result: Box::new(VerificationResult::new()),
        })
    }

    /// Run the test.
    pub fn run(&mut self, directory: &VerificationDirectory) {
        self.status = VerificationStatus::Running;
        let start_time = SystemTime::now();
        info!(
            "Verification {} ({}) started",
            self.meta_data.name, self.meta_data.id
        );
        (self.verification_fn)(directory, self.result.as_mut());
        self.duration = Some(start_time.elapsed().unwrap());
        self.status = VerificationStatus::Finished;
        if self.is_ok().unwrap() {
            info!(
                "Verification {} ({}) finished successfully. Duration: {}s",
                self.meta_data.name,
                self.meta_data.id,
                self.duration.unwrap().as_secs_f32()
            );
        }
        if self.has_errors().unwrap() {
            warn!(
                "Verification {} ({}) finished with errors. Duration: {}s",
                self.meta_data.name,
                self.meta_data.id,
                self.duration.unwrap().as_secs_f32()
            );
        }
        if self.has_failures().unwrap() {
            warn!(
                "Verification {} ({}) finished with failures. Duration: {}s",
                self.meta_data.name,
                self.meta_data.id,
                self.duration.unwrap().as_secs_f32()
            );
        }
    }
}

impl VerificationResult {
    /// New VerificationResult
    pub fn new() -> Self {
        VerificationResult {
            errors: vec![],
            failures: vec![],
        }
    }

    /// Mutable reference to the errors
    fn errors_mut(&mut self) -> &mut Vec<VerificationError> {
        &mut self.errors
    }

    /// Mutable reference to the failures
    fn failures_mut(&mut self) -> &mut Vec<VerificationFailure> {
        &mut self.failures
    }

    /// Push a new error to the VerificationResult
    pub fn push_error(&mut self, e: VerificationError) {
        self.errors.push(e)
    }

    /// Push a new failure to the VerificationResult
    pub fn push_failure(&mut self, f: VerificationFailure) {
        self.failures.push(f)
    }

    /// Append the results of ohter to self, emptying the vectors of other
    pub fn append(&mut self, other: &mut Self) {
        self.errors.append(&mut other.errors_mut());
        self.failures.append(&mut other.failures_mut());
    }
}

impl VerificationResultTrait for VerificationResult {
    fn is_ok(&self) -> Option<bool> {
        Some(!self.has_errors().unwrap() && !self.has_failures().unwrap())
    }

    fn has_errors(&self) -> Option<bool> {
        Some(!self.errors.is_empty())
    }

    fn has_failures(&self) -> Option<bool> {
        Some(!self.failures.is_empty())
    }

    fn errors(&self) -> &Vec<VerificationError> {
        &self.errors
    }

    fn failures(&self) -> &Vec<VerificationFailure> {
        &self.failures
    }
}

impl<'a> VerificationResultTrait for Verification<'a, VerificationDirectory> {
    fn is_ok(&self) -> Option<bool> {
        match self.status {
            VerificationStatus::Stopped => None,
            VerificationStatus::Running => None,
            VerificationStatus::Finished => self.result.is_ok(),
        }
    }

    fn has_errors(&self) -> Option<bool> {
        match self.status {
            VerificationStatus::Stopped => None,
            VerificationStatus::Running => None,
            VerificationStatus::Finished => self.result.has_errors(),
        }
    }

    fn has_failures(&self) -> Option<bool> {
        match self.status {
            VerificationStatus::Stopped => None,
            VerificationStatus::Running => None,
            VerificationStatus::Finished => self.result.has_failures(),
        }
    }

    fn errors(&self) -> &Vec<VerificationError> {
        self.result.errors()
    }

    fn failures(&self) -> &Vec<VerificationFailure> {
        self.result.failures()
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::error::{create_verifier_error, VerifierError};
    use crate::verification::error::{VerificationErrorType, VerificationFailureType};
    use crate::verification::VerificationPeriod;

    use super::*;

    #[test]
    fn run_ok() {
        fn ok(_: &VerificationDirectory, _: &mut VerificationResult) {}
        let md_list = VerificationMetaDataList::load().unwrap();
        let mut verif = Verification::new("s100", ok, &md_list).unwrap();
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(verif.is_ok().is_none());
        assert!(verif.has_errors().is_none());
        assert!(verif.has_failures().is_none());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            &Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Finished);
        assert!(verif.is_ok().unwrap());
        assert!(!verif.has_errors().unwrap());
        assert!(!verif.has_failures().unwrap());
    }

    #[test]
    fn run_error() {
        fn error(_: &VerificationDirectory, result: &mut VerificationResult) {
            result.push_error(create_verifier_error!(VerificationErrorType::Error, "toto"));
            result.push_error(create_verifier_error!(
                VerificationErrorType::Error,
                "toto2"
            ));
            result.push_failure(create_verifier_error!(
                VerificationFailureType::Failure,
                "toto"
            ));
        }
        let md_list = VerificationMetaDataList::load().unwrap();
        let mut verif = Verification::new("s100", error, &md_list).unwrap();
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(verif.is_ok().is_none());
        assert!(verif.has_errors().is_none());
        assert!(verif.has_failures().is_none());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            &Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Finished);
        assert!(!verif.is_ok().unwrap());
        assert!(verif.has_errors().unwrap());
        assert!(verif.has_failures().unwrap());
        assert_eq!(verif.errors().len(), 2);
        assert_eq!(verif.failures().len(), 1);
    }

    #[test]
    fn run_failure() {
        fn failure(_: &VerificationDirectory, result: &mut VerificationResult) {
            result.push_failure(create_verifier_error!(
                VerificationFailureType::Failure,
                "toto"
            ));
            result.push_failure(create_verifier_error!(
                VerificationFailureType::Failure,
                "toto2"
            ));
        }
        let md_list = VerificationMetaDataList::load().unwrap();
        let mut verif = Verification::new("s100", failure, &md_list).unwrap();
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(verif.is_ok().is_none());
        assert!(verif.has_errors().is_none());
        assert!(verif.has_failures().is_none());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            &Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Finished);
        assert!(!verif.is_ok().unwrap());
        assert!(!verif.has_errors().unwrap());
        assert!(verif.has_failures().unwrap());
        assert_eq!(verif.errors().len(), 0);
        assert_eq!(verif.failures().len(), 2);
    }
}
