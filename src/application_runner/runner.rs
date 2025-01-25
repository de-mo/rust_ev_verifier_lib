//use futures::{stream::FuturesUnordered, StreamExt};
use crate::{
    application_runner::checks::{check_complete, check_verification_dir},
    config::VerifierConfig,
    file_structure::{VerificationDirectory, VerificationDirectoryTrait},
    verification::{
        VerificationMetaDataList, VerificationPeriod, VerificationStatus, VerificationSuite,
    },
};
use tracing::{info, warn};
//use std::future::Future;
use super::{checks::start_check, prepare_fixed_based_optimization, RunnerError};
use rayon::prelude::*;
use std::{iter::zip, sync::Mutex};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

pub fn no_action_before_runner_fn(_: SystemTime) {}
pub fn no_action_before_fn(_: &str) {}
pub fn no_action_after_fn(_: VerificationRunInformation) {}
pub fn no_action_after_runner_fn(_: RunnerInformation) {}

/// Srtucture to collect the information of the run of a verification
pub struct VerificationRunInformation {
    /// id of the verification
    pub id: String,
    /// status of the verification
    pub status: VerificationStatus,
    /// List of failures as [String]
    pub failures: Vec<String>,
    /// List of errors as [String]
    pub errors: Vec<String>,
}

/// Information of the runner, that can be used to know some information about the runner.
#[derive(Debug, Clone, Default)]
pub struct RunnerInformation {
    pub start_time: Option<SystemTime>,
    pub duration: Option<Duration>,
}

impl RunnerInformation {
    pub fn is_finished(&self) -> bool {
        self.duration.is_some()
    }

    pub fn is_running(&self) -> bool {
        self.start_time.is_some() && self.duration.is_none()
    }
}

/// Strategy to run the tests
pub trait RunStrategy<'a> {
    /// Run function
    ///
    /// - `verifications`: The suite of verifications, which will be modified during the run
    /// - `directory`: Verification directoy containing the datasets extracted
    /// - `action_before_verification`:
    ///     Function that will be call before the run of each verification. As parameter take the id of the verification
    /// - `action_after_verification`:
    ///     Function that will be call before the run of each verification.
    ///     As parameter take the information regarding the run of the verification
    fn run(
        &self,
        verifications: &'a mut VerificationSuite<'a>,
        directory: &VerificationDirectory,
        action_before_verification: impl Fn(&str) + Send + Sync,
        action_after_verification: impl Fn(VerificationRunInformation) + Send + Sync,
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
        action_before_verification: impl Fn(&str) + Send + Sync,
        action_after_verification: impl Fn(VerificationRunInformation) + Send + Sync,
    ) {
        let it = verifications.verifications_mut().0.iter_mut();
        for v in it {
            action_before_verification(v.id());
            v.run(directory);
            action_after_verification(VerificationRunInformation {
                id: v.id().to_string(),
                status: v.status(),
                failures: v.verification_result().errors_to_string(),
                errors: v.verification_result().failures_to_string(),
            });
        }
    }
}

impl<'a> RunStrategy<'a> for RunParallel {
    fn run(
        &self,
        verifications: &'a mut VerificationSuite<'a>,
        directory: &VerificationDirectory,
        action_before_verification: impl Fn(&str) + Send + Sync,
        action_after_verification: impl Fn(VerificationRunInformation) + Send + Sync,
    ) {
        let dirs = vec![directory; verifications.len()];
        zip(
            verifications
                .verifications_mut()
                .0
                .iter_mut()
                .map(Mutex::new),
            dirs,
        )
        .par_bridge()
        .for_each(|(vm, d)| {
            let mut v = vm.lock().unwrap();
            action_before_verification(v.id());
            v.run(d);
            action_after_verification(VerificationRunInformation {
                id: v.id().to_string(),
                status: v.status(),
                failures: v.verification_result().errors_to_string(),
                errors: v.verification_result().failures_to_string(),
            });
        });
    }
}

/// Structure defining the runner
///
/// The runner can run only once. The runner has to be reseted to restart.
pub struct Runner<'a, T: RunStrategy<'a>> {
    path: PathBuf,
    verification_directory: Box<VerificationDirectory>,
    verifications: Box<VerificationSuite<'a>>,
    start_time: Option<SystemTime>,
    duration: Option<Duration>,
    run_strategy: T,
    config: &'static VerifierConfig,
    action_before_runner: Box<dyn Fn(SystemTime) + Send + Sync>,
    action_before_verification: Box<dyn Fn(&str) + Send + Sync>,
    #[allow(clippy::type_complexity)]
    action_after_verification: Box<dyn Fn(VerificationRunInformation) + Send + Sync>,
    action_after_runner: Box<dyn Fn(RunnerInformation) + Send + Sync>,
}

impl<'a, T> Runner<'a, T>
where
    T: RunStrategy<'a>,
{
    /// Create a new runner.
    ///
    /// - `path` represents the location where the directory setup and tally are stored
    /// - `period` is the verification period
    /// - `metadata`: The list of the metadata of the verifications
    /// - `exclusion`: The list of verifications excluded (list of ids)
    /// - `run_strategy`: The choosen run strategy
    /// - `config`: The configuration of the verifier
    /// - `action_before_verification`:
    ///     Function that will be call before the run of each verification. As parameter take the id of the verification
    /// - `action_after_verification`:
    ///     Function that will be call before the run of each verification.
    ///     As parameter take the information regarding the run of the verification
    ///
    /// It is recommended to keep the data in the calling function and to update them in the closure as follow:
    /// ```ignored
    /// let verifications_not_finished = Arc::new(RwLock::new(vec![]));
    /// let verifications_not_finished_cloned = verifications_not_finished.clone();
    /// let verifications_with_errors_and_failures = Arc::new(RwLock::new(HashMap::new()));
    /// let runner_information = Arc::new(RwLock::new(RunnerInformation::default()));
    /// let mut runner = Runner::new(
    ///     extracted.location(),
    ///     period,
    ///     &metadata,
    ///     exclusion.as_slice(),
    ///     RunParallel,
    ///     config,
    ///     move |id| {
    ///         let mut verif_not_finished_mut = verifications_not_finished.write().unwrap();
    ///         verif_not_finished_mut.push(id.to_string());
    ///     },
    ///     move |verif_information| {
    ///         let mut verif_not_finished_mut = verifications_not_finished_cloned.write().unwrap();
    ///         match verif_not_finished_mut
    ///             .iter()
    ///             .position(|id| id == &verif_information.id)
    ///         {
    ///             Some(pos) => {
    ///                 let _ = verif_not_finished_mut.remove(pos);
    ///             }
    ///             None => {}
    ///         }
    ///         if verif_information.status != VerificationStatus::FinishedSuccessfully {
    ///             let mut verifs_res_mut = verifications_with_errors_and_failures.write().unwrap();
    ///             verifs_res_mut.insert(
    ///                 verif_information.id.clone(),
    ///                 (
    ///                     verif_information.errors.len() as u8,
    ///                     verif_information.failures.len() as u8,
    ///                 ),
    ///             );
    ///         }
    ///     },
    ///     move |run_info| {
    ///         let mut r_info_mut = runner_information.write().unwrap();
    ///         r_info_mut.start_time = run_info.start_time.clone();
    ///         r_info_mut.duration = run_info.duration;
    ///     },
    /// )
    /// ```
    ///
    /// Comments to the above example:
    /// - The data used [Arc] and [RwLock] in order to ensure the thread safety and clone
    /// - The clone of verifications_not_finished is the avoid an error having the lock of the value in both methods, then in the same thread (see `write` of [RwLock])
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: &Path,
        period: &VerificationPeriod,
        metadata: &'a VerificationMetaDataList,
        exclusion: &[String],
        run_strategy: T,
        config: &'static VerifierConfig,
        action_before_runner: impl Fn(SystemTime) + Send + Sync + 'static,
        action_before_verification: impl Fn(&str) + Send + Sync + 'static,
        action_after_verification: impl Fn(VerificationRunInformation) + Send + Sync + 'static,
        action_after_runner: impl Fn(RunnerInformation) + Send + Sync + 'static,
    ) -> Result<Runner<'a, T>, RunnerError> {
        start_check(config).map_err(RunnerError::CheckError)?;
        check_verification_dir(period, path).map_err(RunnerError::CheckError)?;
        let directory = VerificationDirectory::new(period, path);
        check_complete(period, &directory).map_err(RunnerError::CheckError)?;
        prepare_fixed_based_optimization(&directory)?;
        Ok(Runner {
            path: path.to_path_buf(),
            verification_directory: Box::new(directory),
            verifications: Box::new(
                VerificationSuite::new(period, metadata, exclusion, config)
                    .map_err(RunnerError::Verification)?,
            ),
            start_time: None,
            duration: None,
            run_strategy,
            config,
            action_before_runner: Box::new(action_before_runner),
            action_before_verification: Box::new(action_before_verification),
            action_after_verification: Box::new(action_after_verification),
            action_after_runner: Box::new(action_after_runner),
        })
    }

    /// Reset the verifications
    pub fn reset(
        &'a mut self,
        metadata_list: &'a VerificationMetaDataList,
    ) -> Result<(), RunnerError> {
        self.start_time = None;
        self.duration = None;
        self.verifications = Box::new(
            VerificationSuite::new(
                self.period(),
                metadata_list,
                self.verifications.exclusion(),
                self.config,
            )
            .map_err(RunnerError::Verification)?,
        );
        Ok(())
    }

    /// Run all tests
    pub fn run_all<'c: 'a>(
        &'c mut self,
        metadata_list: &'a VerificationMetaDataList,
    ) -> Result<(), RunnerError> {
        if self.is_running() {
            return Err(RunnerError::IsRunning);
        }
        if self.is_finished() {
            return Err(RunnerError::HasAlreadyRun);
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
        (self.action_before_runner)(self.start_time.unwrap());
        {
            self.run_strategy.run(
                &mut self.verifications,
                &self.verification_directory,
                &self.action_before_verification,
                &self.action_after_verification,
            );
        }
        self.duration = Some(self.start_time.unwrap().elapsed().unwrap());
        (self.action_after_runner)(RunnerInformation {
            start_time: self.start_time,
            duration: self.duration,
        });
        info!(
            "{} verifications run (duration: {}s)",
            &len,
            self.duration.unwrap().as_secs_f32()
        );
        Ok(())
    }

    pub fn verifications(&'a self) -> &'a VerificationSuite<'a> {
        &self.verifications
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn verification_directory_path(&self) -> &Path {
        self.verification_directory.path()
    }

    pub fn verification_directory(&self) -> &VerificationDirectory {
        &self.verification_directory
    }

    pub fn start_time(&self) -> Option<SystemTime> {
        self.start_time
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn stop_time(&self) -> Option<SystemTime> {
        if self.start_time.is_some() && self.duration.is_some() {
            return Some(self.start_time.unwrap() + self.duration.unwrap());
        }
        None
    }
}
