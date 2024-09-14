use anyhow::bail;
//use futures::{stream::FuturesUnordered, StreamExt};
use crate::{
    application_runner::checks::{check_complete, check_verification_dir},
    config::Config as VerifierConfig,
    file_structure::VerificationDirectory,
    verification::{
        meta_data::VerificationMetaDataList, suite::VerificationSuite, VerificationPeriod,
    },
};
use tracing::{info, warn};
//use std::future::Future;
use super::checks::start_check;
use rayon::prelude::*;
use std::{iter::zip, sync::Mutex};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

pub fn no_action_before_fn(_: &str) {}
pub fn no_action_after_fn(_: &str, _: Vec<String>, _: Vec<String>) {}

/// Strategy to run the tests
pub trait RunStrategy<'a> {
    /// Run function
    fn run(
        &self,
        verifications: &'a mut VerificationSuite<'a>,
        directory: &VerificationDirectory,
        action_before: impl Fn(&str) + Send + Sync,
        action_after: impl Fn(&str, Vec<String>, Vec<String>) + Send + Sync,
    );
}

/// Strategy to run the tests sequentially
pub struct RunSequential;

/// Strategy to run the tests concurrently
pub struct RunParallel;

impl<'a> RunStrategy<'a> for RunSequential {
    fn run(
        &self,
        verifications: &'a mut VerificationSuite<'a>,
        directory: &VerificationDirectory,
        action_before: impl Fn(&str) + Send + Sync,
        action_after: impl Fn(&str, Vec<String>, Vec<String>) + Send + Sync,
    ) {
        let it = verifications.list.0.iter_mut();
        for v in it {
            action_before(v.id());
            v.run(directory);
            action_after(
                v.id(),
                v.verification_result().errors_to_string(),
                v.verification_result().failures_to_string(),
            );
        }
    }
}

impl<'a> RunStrategy<'a> for RunParallel {
    fn run(
        &self,
        verifications: &'a mut VerificationSuite<'a>,
        directory: &VerificationDirectory,
        action_before: impl Fn(&str) + Send + Sync,
        action_after: impl Fn(&str, Vec<String>, Vec<String>) + Send + Sync,
    ) {
        let dirs = vec![directory; verifications.len()];
        zip(verifications.list.0.iter_mut().map(Mutex::new), dirs)
            .par_bridge()
            .for_each(|(vm, d)| {
                let mut v = vm.lock().unwrap();
                action_before(v.id());
                v.run(d);
                action_after(
                    v.id(),
                    v.verification_result().errors_to_string(),
                    v.verification_result().failures_to_string(),
                );
            });
    }
}

/// Structure defining the runner
///
/// The runner can run only once. The runner has to be reseted to restart.
pub struct Runner<'a, T: RunStrategy<'a>> {
    path: PathBuf,
    verification_directory: VerificationDirectory,
    verifications: Box<VerificationSuite<'a>>,
    start_time: Option<SystemTime>,
    duration: Option<Duration>,
    run_strategy: T,
    #[allow(dead_code)]
    config: &'static VerifierConfig,
    action_before: Box<dyn Fn(&str) + Send + Sync>,
    #[allow(clippy::type_complexity)]
    action_after: Box<dyn Fn(&str, Vec<String>, Vec<String>) + Send + Sync>,
}

impl<'a, T> Runner<'a, T>
where
    T: RunStrategy<'a>,
{
    /// Create a new runner.
    ///
    /// path represents the location where the directory setup and tally are stored
    /// period ist the verification period
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: &Path,
        period: &VerificationPeriod,
        metadata: &'a VerificationMetaDataList,
        exclusion: &[String],
        run_strategy: T,
        config: &'static VerifierConfig,
        action_before: impl Fn(&str) + Send + Sync + 'static,
        action_after: impl Fn(&str, Vec<String>, Vec<String>) + Send + Sync + 'static,
    ) -> anyhow::Result<Runner<'a, T>> {
        start_check(config)?;
        check_verification_dir(period, path)?;
        let directory = VerificationDirectory::new(period, path);
        check_complete(period, &directory)?;
        Ok(Runner {
            path: path.to_path_buf(),
            verification_directory: directory,
            verifications: Box::new(VerificationSuite::new(period, metadata, exclusion, config)?),
            start_time: None,
            duration: None,
            run_strategy,
            config,
            action_before: Box::new(action_before),
            action_after: Box::new(action_after),
        })
    }

    /// Reset the verifications
    #[allow(dead_code)]
    pub fn reset(&'a mut self, metadata_list: &'a VerificationMetaDataList) -> anyhow::Result<()> {
        self.start_time = None;
        self.duration = None;
        self.verifications = Box::new(VerificationSuite::new(
            self.period(),
            metadata_list,
            self.verifications.exclusion(),
            self.config,
        )?);
        Ok(())
    }

    /// Run all tests
    pub fn run_all<'c: 'a>(
        &'c mut self,
        metadata_list: &'a VerificationMetaDataList,
    ) -> anyhow::Result<()> {
        if self.is_running() {
            bail!(format!("Runner is already running. Cannot be started"));
        }
        if self.is_finished() {
            bail!(format!(
                "Runner is already running. Cannot be started before resetting it"
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
                metadata_list.meta_data_from_id(id).unwrap().name(),
                id
            );
        }
        let len = self.verifications.len();
        {
            self.run_strategy.run(
                &mut self.verifications,
                &self.verification_directory,
                &self.action_before,
                &self.action_after,
            );
        }
        self.duration = Some(self.start_time.unwrap().elapsed().unwrap());
        info!(
            "{} verifications run (duration: {}s)",
            &len,
            self.duration.unwrap().as_secs_f32()
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub fn verifications_mut(&'a mut self) -> &'a mut VerificationSuite<'a> {
        &mut self.verifications
    }

    pub fn is_finished(&self) -> bool {
        self.duration.is_some()
    }

    pub fn is_running(&self) -> bool {
        self.start_time.is_some() && self.duration.is_none()
    }

    #[allow(dead_code)]
    pub fn can_be_started(&self) -> bool {
        self.start_time.is_none()
    }

    pub fn period(&self) -> &VerificationPeriod {
        self.verifications.period()
    }
}
