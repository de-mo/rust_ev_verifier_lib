//! Module implementing common functionalities for all Verifier applications (console and GUI)

mod checks;
mod extract;
mod report;
mod runner;

pub use checks::*;
pub use extract::*;
pub use report::*;
pub use runner::{no_action_after_fn, no_action_before_fn, RunParallel, Runner};

use thiserror::Error;

use crate::{dataset::DatasetError, verification::VerificationError};

// Enum representing the datza structure errors
#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("IO error {msg} -> caused by: {source}")]
    IO { msg: String, source: std::io::Error },
    #[error("Check error: {0}")]
    CheckError(String),
    #[error(transparent)]
    Verification(VerificationError),
    #[error("Runner is already running. Cannot be started")]
    IsRunning,
    #[error("Runner has already run. Cannot be started before resetting it")]
    HasAlreadyRun,
    #[error("IO error {msg} -> caused by: {source}")]
    Dataset { msg: String, source: DatasetError },
    #[error("{0}")]
    FileMissing(String),
}
