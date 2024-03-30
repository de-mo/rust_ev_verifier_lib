//! Module implementing the structure of a verification
use super::{
    meta_data::{VerificationMetaData, VerificationMetaDataList},
    result::{VerificationEvent, VerificationResult, VerificationResultTrait},
    VerificationStatus,
};
use crate::{
    config::Config,
    file_structure::{VerificationDirectory, VerificationDirectoryTrait},
};
use anyhow::bail;
use log::{info, warn};
use std::time::{Duration, SystemTime};

/// Struct representing a verification
#[allow(clippy::type_complexity)]
pub struct Verification<'a, D: VerificationDirectoryTrait> {
    /// Id of the verification
    id: String,
    /// Metadata of the verification
    ///
    /// The meta data is a reference to the metadata list loaded from json
    meta_data: &'a VerificationMetaData,
    status: VerificationStatus,
    verification_fn: Box<dyn Fn(&D, &'static Config, &mut VerificationResult) + Send + Sync>,
    duration: Option<Duration>,
    result: Box<VerificationResult>,
    config: &'static Config,
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
        verification_fn: impl Fn(&VerificationDirectory, &'static Config, &mut VerificationResult)
            + Send
            + Sync
            + 'static,
        metadata_list: &'a VerificationMetaDataList,
        config: &'static Config,
    ) -> anyhow::Result<Self> {
        let meta_data = match metadata_list.meta_data_from_id(id) {
            Some(m) => m,
            None => {
                bail!(format!("metadata for verification id {} not found", id))
            }
        };
        if name != meta_data.name() {
            bail!(format!(
                "name {} for verification id {} doesn't match with give name {}",
                meta_data.name(),
                id,
                name
            ))
        }
        Ok(Verification {
            id: id.to_string(),
            meta_data,
            status: VerificationStatus::Stopped,
            verification_fn: Box::new(verification_fn),
            duration: None,
            result: Box::new(VerificationResult::new()),
            config,
        })
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    #[allow(dead_code)]
    pub fn meta_data(&'a self) -> &'a VerificationMetaData {
        self.meta_data
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
        self.status = VerificationStatus::Finished;
        if self.is_ok() {
            info!(
                "Verification {} ({}) finished successfully. Duration: {}s",
                self.meta_data.name(),
                self.meta_data.id(),
                self.duration.unwrap().as_secs_f32()
            );
        }
        if self.has_errors() {
            warn!(
                "Verification {} ({}) finished with errors. Duration: {}s",
                self.meta_data.name(),
                self.meta_data.id(),
                self.duration.unwrap().as_secs_f32()
            );
        }
        if self.has_failures() {
            warn!(
                "Verification {} ({}) finished with failures. Duration: {}s",
                self.meta_data.name(),
                self.meta_data.id(),
                self.duration.unwrap().as_secs_f32()
            );
        }
    }
}

impl<'a> VerificationResultTrait for Verification<'a, VerificationDirectory> {
    fn is_valid(&self) -> bool {
        match self.status {
            VerificationStatus::Stopped => false,
            VerificationStatus::Running => false,
            VerificationStatus::Finished => true,
        }
    }
    fn is_ok(&self) -> bool {
        self.is_valid() && self.result.is_ok()
    }

    fn has_errors(&self) -> bool {
        self.is_valid() && self.result.has_errors()
    }

    fn has_failures(&self) -> bool {
        self.is_valid() && self.result.has_failures()
    }

    fn errors(&self) -> &Vec<VerificationEvent> {
        self.result.errors()
    }

    fn failures(&self) -> &Vec<VerificationEvent> {
        self.result.failures()
    }

    fn errors_mut(&mut self) -> &mut Vec<VerificationEvent> {
        self.result.errors_mut()
    }

    fn failures_mut(&mut self) -> &mut Vec<VerificationEvent> {
        self.result.failures_mut()
    }

    fn errors_to_string(&self) -> Vec<String> {
        self.result.errors_to_string()
    }

    fn failures_to_string(&self) -> Vec<String> {
        self.result.failures_to_string()
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::{
            result::{create_verification_error, create_verification_failure},
            VerificationPeriod,
        },
        *,
    };
    use crate::config::test::CONFIG_TEST;
    use anyhow::anyhow;
    use log::debug;
    use std::path::Path;

    #[test]
    fn test_creation() {
        fn ok(_: &VerificationDirectory, _: &'static Config, _: &mut VerificationResult) {}
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
        fn ok(_: &VerificationDirectory, _: &'static Config, _: &mut VerificationResult) {}
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
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(!verif.is_valid());
        assert!(!verif.is_ok());
        assert!(!verif.has_errors());
        assert!(!verif.has_failures());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Finished);
        assert!(verif.is_valid());
        assert!(verif.is_ok());
        assert!(!verif.has_errors());
        assert!(!verif.has_failures());
    }

    #[test]
    fn run_error() {
        fn error(_: &VerificationDirectory, _: &'static Config, result: &mut VerificationResult) {
            result.push(create_verification_error!("toto"));
            result.push(create_verification_error!("toto2"));
            result.push(create_verification_failure!("toto3"));
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
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(!verif.is_valid());
        assert!(!verif.is_ok());
        assert!(!verif.has_errors());
        assert!(!verif.has_failures());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Finished);
        assert!(verif.is_valid());
        assert!(!verif.is_ok());
        assert!(verif.has_errors());
        assert!(verif.has_failures());
        assert_eq!(verif.errors().len(), 2);
        assert_eq!(verif.failures().len(), 1);
    }

    #[test]
    fn run_failure() {
        fn failure(_: &VerificationDirectory, _: &'static Config, result: &mut VerificationResult) {
            result.push(create_verification_failure!("toto"));
            result.push(create_verification_failure!("toto2"));
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
        assert_eq!(verif.status, VerificationStatus::Stopped);
        assert!(!verif.is_valid());
        assert!(!verif.is_ok());
        assert!(!verif.has_errors());
        assert!(!verif.has_failures());
        verif.run(&VerificationDirectory::new(
            &VerificationPeriod::Setup,
            Path::new("."),
        ));
        assert_eq!(verif.status, VerificationStatus::Finished);
        assert!(verif.is_valid());
        assert!(!verif.is_ok());
        assert!(!verif.has_errors());
        assert!(verif.has_failures());
        assert_eq!(verif.errors().len(), 0);
        assert_eq!(verif.failures().len(), 2);
    }
}
