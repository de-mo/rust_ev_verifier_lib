//! Module implementing the runner

use super::error::{create_verifier_error, VerifierError};
use crate::{
    file_structure::VerificationDirectory,
    verification::{
        meta_data::{VerificationMetaDataList, VerificationMetaDataListTrait},
        verification_suite::VerificationSuite,
        VerificationPeriod,
    },
};
use log::{info, warn};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

/// Structure defining the runner
///
/// The runner can run only once. The runner has to be reseted to restart.
pub struct Runner {
    path: PathBuf,
    verifications: VerificationSuite,
    start_time: Option<SystemTime>,
    duration: Option<Duration>,
}

impl Runner {
    /// Create a new runner.
    ///
    /// path represents the location where the directory setup and tally are stored
    /// period ist the verification period
    pub fn new(
        path: &Path,
        period: &VerificationPeriod,
        metadata: &VerificationMetaDataList,
        exclusion: &Option<Vec<String>>,
    ) -> Runner {
        Runner {
            path: path.to_path_buf(),
            verifications: VerificationSuite::new(period, metadata, exclusion),
            start_time: None,
            duration: None,
        }
    }

    /// Reset the verifications
    pub fn reset(&mut self, metadata_list: &VerificationMetaDataList) {
        self.start_time = None;
        self.duration = None;
        self.verifications = VerificationSuite::new(
            self.period(),
            metadata_list,
            &Some(self.verifications.exclusion().clone()),
        )
    }

    /// Run all tests sequentially
    pub fn run_all_sequential(
        &mut self,
        metadata_list: &VerificationMetaDataList,
    ) -> Option<RunnerError> {
        if self.is_running() {
            return Some(create_verifier_error!(
                RunnerErrorType::RunError,
                "Runner is already running. Cannot be started"
            ));
        }
        if self.is_finished() {
            return Some(create_verifier_error!(
                RunnerErrorType::RunError,
                "Runner has already run. Cannot be started before resetting it"
            ));
        }
        self.start_time = Some(SystemTime::now());
        info!(
            "Start all verifications ({} verifications; {} excluded)",
            self.verifications.len(),
            self.verifications.len_excluded()
        );
        for id in self.verifications.exclusion().iter() {
            warn!(
                "Verification {} ({}) skipped",
                metadata_list.meta_data_from_id(id).unwrap().name,
                id
            )
        }
        let directory = VerificationDirectory::new(self.period(), &self.path);
        for v in self.verifications.verifications_mut().iter_mut() {
            v.run(&directory);
        }
        self.duration = Some(self.start_time.unwrap().elapsed().unwrap());
        info!(
            "{} verifications run (duration: {}s)",
            self.verifications.len(),
            self.duration.unwrap().as_secs_f32()
        );
        None
    }

    pub fn is_finished(&self) -> bool {
        self.duration.is_some()
    }

    pub fn is_running(&self) -> bool {
        self.start_time.is_some() && self.duration.is_none()
    }

    pub fn can_be_started(&self) -> bool {
        self.start_time.is_none()
    }

    pub fn period(&self) -> &VerificationPeriod {
        self.verifications.period()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerErrorType {
    RunError,
}

impl Display for RunnerErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::RunError => "RunError",
        };
        write!(f, "{s}")
    }
}

pub type RunnerError = VerifierError<RunnerErrorType>;
