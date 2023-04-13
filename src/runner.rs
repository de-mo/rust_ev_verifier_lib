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
pub struct Runner<'a> {
    path: PathBuf,
    verifications: Box<VerificationSuite<'a>>,
    start_time: Option<SystemTime>,
    duration: Option<Duration>,
}

impl<'a> Runner<'a> {
    /// Create a new runner.
    ///
    /// path represents the location where the directory setup and tally are stored
    /// period ist the verification period
    pub fn new(
        path: &Path,
        period: &VerificationPeriod,
        metadata: &'a VerificationMetaDataList,
        exclusion: &Option<Vec<String>>,
    ) -> Runner<'a> {
        Runner {
            path: path.to_path_buf(),
            verifications: Box::new(VerificationSuite::new(period, metadata, exclusion)),
            start_time: None,
            duration: None,
        }
    }

    /// Reset the verifications
    pub fn reset(&'a mut self, metadata_list: &'a VerificationMetaDataList) {
        self.start_time = None;
        self.duration = None;
        self.verifications = Box::new(VerificationSuite::new(
            self.period(),
            metadata_list,
            &Some(self.verifications.exclusion().clone()),
        ))
    }

    /// Run all tests sequentially
    pub fn run_all_sequential<'b: 'a>(
        &'b mut self,
        metadata_list: &'a VerificationMetaDataList,
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
        let it = self.verifications.list.iter_mut();
        for v in it {
            v.run(&directory);
        }
        self.duration = Some(self.start_time.unwrap().elapsed().unwrap());
        info!(
            "{} verifications run (duration: {}s)",
            &self.verifications.len(),
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
