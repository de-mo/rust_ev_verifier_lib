use std::{collections::HashMap, path::Path, time::SystemTime};

use super::{
    runner::VerificationRunInformation, ExtractDataSetResults, RunnerError, RunnerInformation,
};
use crate::{
    file_structure::VerificationDirectory,
    verification::{
        get_verifications_setup, get_verifications_tally, ManualVerifications,
        VerificationMetaDataList, VerificationPeriod, VerificationStatus,
    },
    VerifierConfig,
};

/// This structure contains all the information about the actual run.
///
/// It helps, the collect the necessary information outside the runner, and to avoid the borrow of mutable borrow.
pub struct RunInformation {
    config: &'static VerifierConfig,
    verification_period: Option<VerificationPeriod>,
    extracted_dataset_result: Option<ExtractDataSetResults>,
    verification_metadata: Option<VerificationMetaDataList>,
    excluded_verifications: Vec<String>,
    verifications_status: HashMap<String, VerificationStatus>,
    verifications_with_errors_and_failures: HashMap<String, (Vec<String>, Vec<String>)>,
    runner_information: RunnerInformation,
}

impl RunInformation {
    /// New [RunInformation] containung the config
    pub fn new(config: &'static VerifierConfig) -> Self {
        Self {
            config,
            verification_period: None,
            extracted_dataset_result: None,
            verification_metadata: None,
            verifications_status: HashMap::default(),
            excluded_verifications: vec![],
            verifications_with_errors_and_failures: HashMap::default(),
            runner_information: RunnerInformation::default(),
        }
    }

    /// Prepare the data for running, collecting the necesssary information
    pub fn prepare_data_for_running(
        &mut self,
        verification_period: VerificationPeriod,
        verification_metadata: &VerificationMetaDataList,
        excluded_verifications: &[String],
        extracted_dataset_result: &ExtractDataSetResults,
    ) -> Result<(), RunnerError> {
        self.verification_period = Some(verification_period);
        self.verification_metadata = Some(verification_metadata.clone());
        self.excluded_verifications = excluded_verifications.to_vec();
        self.extracted_dataset_result = Some(extracted_dataset_result.clone());
        let all_verifs = match verification_period {
            VerificationPeriod::Setup => {
                get_verifications_setup(verification_metadata, self.config).map_err(|e| {
                    RunnerError::RunInformationError(format!(
                        "Collecting verifications setup: {}",
                        e
                    ))
                })?
            }
            VerificationPeriod::Tally => {
                get_verifications_tally(verification_metadata, self.config).map_err(|e| {
                    RunnerError::RunInformationError(format!(
                        "Collecting verifications setup: {}",
                        e
                    ))
                })?
            }
        };
        self.verifications_status = all_verifs
            .0
            .iter()
            .map(|v| v.id().to_string())
            .filter(|id| !excluded_verifications.contains(id))
            .map(|id| (id, VerificationStatus::NotStarted))
            .collect();
        Ok(())
    }

    fn update_verif_status(&mut self, id: &str, status: VerificationStatus) {
        if self.verifications_status.contains_key(id) {
            let _ = self.verifications_status.insert(id.to_string(), status);
        }
    }

    /// Update information starting the runner
    pub fn start_running(&mut self, start_time: &SystemTime) {
        self.runner_information.start_time = Some(*start_time)
    }

    /// Update information starting the given verification if the id
    pub fn start_verification(&mut self, id: &str) {
        self.update_verif_status(id, VerificationStatus::Running);
    }

    /// Update information finishing the given verification if the id
    pub fn finish_verification(&mut self, verif_info: &VerificationRunInformation) {
        self.update_verif_status(&verif_info.id, verif_info.status);
        if verif_info.status != VerificationStatus::FinishedSuccessfully {
            self.verifications_with_errors_and_failures.insert(
                verif_info.id.clone(),
                (verif_info.errors.clone(), verif_info.failures.clone()),
            );
        }
    }

    /// Update information finishing the runner
    pub fn finish_runner(&mut self, runner_info: &RunnerInformation) {
        self.runner_information = runner_info.clone()
    }

    /// Are the information in prepared status
    pub fn is_prepared(&self) -> bool {
        !self.verifications_status.is_empty()
    }

    /// Are the information in running status
    pub fn is_running(&self) -> bool {
        self.runner_information().is_running()
    }

    /// Are the information in finished status
    pub fn is_finished(&self) -> bool {
        self.runner_information().is_finished()
    }

    /// Are the information in running or finished status
    pub fn is_running_or_finished(&self) -> bool {
        self.is_running() || self.is_finished()
    }

    /// Configuration of the verifier
    pub fn config(&self) -> &'static VerifierConfig {
        self.config
    }

    /// Verification period
    pub fn verification_period(&self) -> Option<VerificationPeriod> {
        self.verification_period
    }

    /// Extracted information results
    pub fn extracted_dataset_result(&self) -> Option<&ExtractDataSetResults> {
        self.extracted_dataset_result.as_ref()
    }

    /// Verification metadata
    pub fn verification_metadata(&self) -> Option<&VerificationMetaDataList> {
        self.verification_metadata.as_ref()
    }

    /// List of ids for excluded verifications
    pub fn excluded_verifications(&self) -> &[String] {
        self.excluded_verifications.as_slice()
    }

    /// List of ids of verifications in the given status
    pub fn verifications_with_status(&self, status: VerificationStatus) -> Vec<&str> {
        self.verifications_status
            .iter()
            .filter(|(_, v)| **v == status)
            .map(|(k, _)| k.as_str())
            .collect()
    }

    /// List of ids of verifications not started
    pub fn verifications_not_started(&self) -> Vec<&str> {
        self.verifications_with_status(VerificationStatus::NotStarted)
    }

    /// List of ids of verifications running
    pub fn verifications_running(&self) -> Vec<&str> {
        self.verifications_with_status(VerificationStatus::Running)
    }

    /// List of ids of verifications
    pub fn verifications(&self) -> Vec<&str> {
        self.verifications_status
            .keys()
            .map(|k| k.as_str())
            .collect()
    }

    /// Hashmap of the verifications having errors and failures
    ///
    /// Key of the [HashMap] is the id of the verification
    /// Value of the [HashMap] is a tuple:
    ///     - First element is the list of errors
    ///     - Seconde element is the list of failures
    pub fn verifications_with_errors_and_failures(
        &self,
    ) -> &HashMap<String, (Vec<String>, Vec<String>)> {
        &self.verifications_with_errors_and_failures
    }

    /// Information about the runner
    pub fn runner_information(&self) -> &RunnerInformation {
        &self.runner_information
    }

    /// The directory where the datasets (decrypted and unzipped) are stored
    pub fn run_directory(&self) -> &Path {
        self.extracted_dataset_result.as_ref().unwrap().location()
    }
}

impl TryFrom<&RunInformation> for ManualVerifications<VerificationDirectory> {
    type Error = RunnerError;

    fn try_from(value: &RunInformation) -> Result<Self, Self::Error> {
        if !value.is_prepared() {
            return Err(RunnerError::RunInformationError(
                "The run information must be prepared".to_string(),
            ));
        }
        let dir = VerificationDirectory::new(
            value.verification_period.as_ref().unwrap(),
            value.run_directory(),
        );
        Self::try_new(
            value.verification_period.unwrap(),
            &dir,
            value.config,
            &value.verifications_status,
            value.verifications_with_errors_and_failures(),
            &value.excluded_verifications,
        )
        .map_err(|e| {
            RunnerError::RunInformationError(format!(
                "Error creating the manual verifications: {}",
                e
            ))
        })
    }
}
