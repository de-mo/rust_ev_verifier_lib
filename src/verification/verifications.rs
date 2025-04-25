// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

//! Module implementing the structure of a verification
use super::{
    meta_data::{VerificationMetaData, VerificationMetaDataList},
    result::VerificationResult,
    VerificationError, VerificationStatus,
};
use crate::{
    config::VerifierConfig,
    file_structure::{VerificationDirectory, VerificationDirectoryTrait},
};
use std::time::{Duration, SystemTime};
use tracing::{error, info, warn};

/// Struct representing a verification
#[allow(clippy::type_complexity)]
pub struct Verification<'a, D: VerificationDirectoryTrait> {
    /// Metadata of the verification
    ///
    /// The meta data is a reference to the metadata list loaded from json
    meta_data: &'a VerificationMetaData,
    status: VerificationStatus,
    verification_fn:
        Box<dyn Fn(&D, &'static VerifierConfig, &mut VerificationResult) + Send + Sync>,
    duration: Option<Duration>,
    result: Box<VerificationResult>,
    config: &'static VerifierConfig,
}

impl<'a> Verification<'a, VerificationDirectory> {
    /// Create a new verification.
    ///
    /// The input are the metadata and the explicit function of the verification. The function
    /// must have the following form:
    /// The verification functions should only defined with the traits as follow
    /// ```ignore
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
        name: &str,
        verification_fn: impl Fn(&VerificationDirectory, &'static VerifierConfig, &mut VerificationResult)
            + Send
            + Sync
            + 'static,
        metadata_list: &'a VerificationMetaDataList,
        config: &'static VerifierConfig,
    ) -> Result<Self, VerificationError> {
        let meta_data = match metadata_list.meta_data_from_id(id) {
            Some(m) => m,
            None => return Err(VerificationError::MetadataNotFound(id.to_string())),
        };
        if name != meta_data.name() {
            return Err(VerificationError::Generic(format!(
                "name {} for verification id {} doesn't match with give name {}",
                meta_data.name(),
                id,
                name
            )));
        }
        Ok(Verification {
            meta_data,
            status: VerificationStatus::NotStarted,
            verification_fn: Box::new(verification_fn),
            duration: None,
            result: Box::new(VerificationResult::new()),
            config,
        })
    }

    /// Get the id of the verification
    pub fn id(&self) -> &str {
        self.meta_data.id()
    }

    /// Get the meta data of the verification
    pub fn meta_data(&'a self) -> &'a VerificationMetaData {
        self.meta_data
    }

    /// Get status
    pub fn status(&self) -> VerificationStatus {
        self.status
    }

    /// Get the result of the verification
    pub fn verification_result(&self) -> &VerificationResult {
        &self.result
    }

    /// `true` if the verification finished
    pub fn is_result_final(&self) -> bool {
        !matches!(
            self.status,
            VerificationStatus::NotStarted | VerificationStatus::Running
        )
    }

    /// Has the result errors
    ///
    /// `None` if the result is not final
    pub fn has_errors(&self) -> Option<bool> {
        match self.is_result_final() {
            true => Some(self.result.has_errors()),
            false => None,
        }
    }

    /// Has the result failures
    ///
    /// `None` if the result is not final
    pub fn has_failures(&self) -> Option<bool> {
        match self.is_result_final() {
            true => Some(self.result.has_failures()),
            false => None,
        }
    }

    /// Is the result ok
    ///
    /// `None` if the result is not final
    pub fn is_ok(&self) -> Option<bool> {
        match self.is_result_final() {
            true => Some(self.result.is_ok()),
            false => None,
        }
    }

    /// Run the test.
    pub fn run(&mut self, directory: &VerificationDirectory) {
        self.status = VerificationStatus::Running;
        let start_time = SystemTime::now();
        info!(
            "Verification {} ({}) started",
            self.meta_data.name(),
            self.meta_data.id()
        );
        (self.verification_fn)(directory, self.config, self.result.as_mut());
        self.duration = Some(start_time.elapsed().unwrap());
        self.status = VerificationStatus::calculate_finished(
            self.result.has_errors(),
            self.result.has_failures(),
        );
        if self.is_ok().unwrap() {
            info!(
                "Verification {} ({}) finished successfully. Duration: {}s",
                self.meta_data.name(),
                self.meta_data.id(),
                self.duration.unwrap().as_secs_f32()
            );
        }
        if self.has_errors().unwrap() {
            error!(
                "Verification {} ({}) finished with errors. Duration: {}s \n{}",
                self.meta_data.name(),
                self.meta_data.id(),
                self.duration.unwrap().as_secs_f32(),
                self.result.errors_to_string().join("\n")
            );
        }
        if self.has_failures().unwrap() {
            warn!(
                "Verification {} ({}) finished with failures. Duration: {}s \n{}",
                self.meta_data.name(),
                self.meta_data.id(),
                self.duration.unwrap().as_secs_f32(),
                self.result.failures_to_string().join("\n")
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        config::test::CONFIG_TEST,
        verification::{result::VerificationEvent, VerificationPeriod},
    };
    use std::path::Path;

    #[test]
    fn test_creation() {
        fn ok(_: &VerificationDirectory, _: &'static VerifierConfig, _: &mut VerificationResult) {}
        let md_list =
            VerificationMetaDataList::load(CONFIG_TEST.get_verification_list_str()).unwrap();
        assert!(Verification::new(
            "01.01",
            "VerifySetupCompleteness",
            ok,
            &md_list,
            &CONFIG_TEST,
        )
        .is_ok());
        assert!(Verification::new(
            "20.01",
            "VerifySetupCompleteness",
            ok,
            &md_list,
            &CONFIG_TEST,
        )
        .is_err());
        assert!(Verification::new("01.01", "Toto", ok, &md_list, &CONFIG_TEST,).is_err());
    }

    #[test]
    fn run_ok() {
        fn ok(_: &VerificationDirectory, _: &'static VerifierConfig, _: &mut VerificationResult) {}
        let md_list =
            VerificationMetaDataList::load(CONFIG_TEST.get_verification_list_str()).unwrap();
        let mut verif = Verification::new(
            "01.01",
            "VerifySetupCompleteness",
            ok,
            &md_list,
            &CONFIG_TEST,
        )
        .unwrap();
        assert_eq!(verif.status, VerificationStatus::NotStarted);
        assert!(!verif.is_result_final());
        assert!(verif.is_ok().is_none());
        assert!(verif.has_errors().is_none());
        assert!(verif.has_failures().is_none());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::FinishedSuccessfully);
        assert!(verif.is_result_final());
        assert!(verif.is_ok().unwrap());
        assert!(!verif.has_errors().unwrap());
        assert!(!verif.has_failures().unwrap());
    }

    #[test]
    fn run_error() {
        fn error(
            _: &VerificationDirectory,
            _: &'static VerifierConfig,
            result: &mut VerificationResult,
        ) {
            result.push(VerificationEvent::new_error("toto"));
            result.push(VerificationEvent::new_error("toto2"));
            result.push(VerificationEvent::new_failure("toto3"));
        }
        let md_list =
            VerificationMetaDataList::load(CONFIG_TEST.get_verification_list_str()).unwrap();
        let mut verif = Verification::new(
            "01.01",
            "VerifySetupCompleteness",
            error,
            &md_list,
            &CONFIG_TEST,
        )
        .unwrap();
        assert_eq!(verif.status, VerificationStatus::NotStarted);
        assert!(!verif.is_result_final());
        assert!(verif.is_ok().is_none());
        assert!(verif.has_errors().is_none());
        assert!(verif.has_failures().is_none());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            Path::new("."),
        ));
        assert_eq!(
            verif.status,
            VerificationStatus::FinishedWithFailuresAndErrors
        );
        assert!(verif.is_result_final());
        assert!(!verif.is_ok().unwrap());
        assert!(verif.has_errors().unwrap());
        assert!(verif.has_failures().unwrap());
        assert_eq!(verif.verification_result().errors().len(), 2);
        assert_eq!(verif.verification_result().failures().len(), 1);
    }

    #[test]
    fn run_failure() {
        fn failure(
            _: &VerificationDirectory,
            _: &'static VerifierConfig,
            result: &mut VerificationResult,
        ) {
            result.push(VerificationEvent::new_failure("toto"));
            result.push(VerificationEvent::new_failure("toto2"));
        }
        let md_list =
            VerificationMetaDataList::load(CONFIG_TEST.get_verification_list_str()).unwrap();
        let mut verif = Verification::new(
            "01.01",
            "VerifySetupCompleteness",
            failure,
            &md_list,
            &CONFIG_TEST,
        )
        .unwrap();
        assert_eq!(verif.status, VerificationStatus::NotStarted);
        assert!(!verif.is_result_final());
        assert!(verif.is_ok().is_none());
        assert!(verif.has_errors().is_none());
        assert!(verif.has_failures().is_none());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::FinishedWithFailures);
        assert!(verif.is_result_final());
        assert!(!verif.is_ok().unwrap());
        assert!(!verif.has_errors().unwrap());
        assert!(verif.has_failures().unwrap());
        assert_eq!(verif.verification_result().errors().len(), 0);
        assert_eq!(verif.verification_result().failures().len(), 2);
    }
}
