//! Crate implementing the runner

mod checks;

pub use checks::*;

use anyhow::anyhow;
use log::{info, warn};
use rust_verifier_lib::{
    config::Config,
    file_structure::VerificationDirectory,
    verification::{
        meta_data::VerificationMetaDataList, suite::VerificationSuite, VerificationPeriod,
    },
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

/// Strategy to run the tests
pub trait RunStrategy<'a> {
    /// Run function
    fn run(&self, verifications: &'a mut VerificationSuite<'a>, dir_path: &Path);
}

/// Strategy to run the tests sequentially
pub struct RunSequential;

impl<'a> RunStrategy<'a> for RunSequential {
    fn run(&self, verifications: &'a mut VerificationSuite<'a>, dir_path: &Path) {
        let directory = VerificationDirectory::new(verifications.period(), dir_path);
        let it = verifications.list.0.iter_mut();
        for v in it {
            v.run(&directory);
        }
    }
}

/// Structure defining the runner
///
/// The runner can run only once. The runner has to be reseted to restart.
pub struct Runner<'a, T: RunStrategy<'a>> {
    path: PathBuf,
    verifications: Box<VerificationSuite<'a>>,
    start_time: Option<SystemTime>,
    duration: Option<Duration>,
    run_strategy: T,
    config: &'static Config,
}

impl<'a, T> Runner<'a, T>
where
    T: RunStrategy<'a>,
{
    /// Create a new runner.
    ///
    /// path represents the location where the directory setup and tally are stored
    /// period ist the verification period
    pub fn new(
        path: &Path,
        period: &VerificationPeriod,
        metadata: &'a VerificationMetaDataList,
        exclusion: &[String],
        run_strategy: T,
        config: &'static Config,
    ) -> Runner<'a, T> {
        Runner {
            path: path.to_path_buf(),
            verifications: Box::new(VerificationSuite::new(period, metadata, exclusion, config)),
            start_time: None,
            duration: None,
            run_strategy,
            config,
        }
    }

    /// Reset the verifications
    pub fn reset(&'a mut self, metadata_list: &'a VerificationMetaDataList) {
        self.start_time = None;
        self.duration = None;
        self.verifications = Box::new(VerificationSuite::new(
            self.period(),
            metadata_list,
            self.verifications.exclusion(),
            self.config,
        ))
    }

    /// Run all tests sequentially
    pub fn run_all<'b: 'a>(
        &'b mut self,
        metadata_list: &'a VerificationMetaDataList,
    ) -> Option<anyhow::Error> {
        if self.is_running() {
            return Some(anyhow!(format!(
                "Runner is already running. Cannot be started"
            )));
        }
        if self.is_finished() {
            return Some(anyhow!(format!(
                "Runner is already running. Cannot be started before resetting it"
            )));
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
                metadata_list.meta_data_from_id(id).unwrap().name(),
                id
            )
        }
        let len = self.verifications.len();
        {
            self.run_strategy.run(&mut self.verifications, &self.path);
        }
        self.duration = Some(self.start_time.unwrap().elapsed().unwrap());
        info!(
            "{} verifications run (duration: {}s)",
            &len,
            self.duration.unwrap().as_secs_f32()
        );
        None
    }

    pub fn verifications_mut(&'a mut self) -> &'a mut VerificationSuite<'a> {
        &mut self.verifications
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
