//! Module implementing common functionalities for all Verifier applications (console and GUI)

mod checks;
mod extract;
pub mod report;
mod runner;

pub use checks::*;
pub use extract::*;
//pub use report::*;
pub use runner::{
    no_action_after_fn, no_action_after_runner_fn, no_action_before_fn, RunParallel, Runner,
    RunnerInformation,
};

use crate::{
    dataset::DatasetError,
    file_structure::{
        ContextDirectoryTrait, FileStructureError, VerificationDirectory,
        VerificationDirectoryTrait,
    },
    verification::VerificationError,
};
use thiserror::Error;

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
    #[error("File structure error {msg} -> caused by: {source}")]
    FileStructure {
        msg: String,
        source: Box<FileStructureError>,
    },
}

fn prepare_fixed_based_optimization(dir: &VerificationDirectory) -> Result<(), RunnerError> {
    let context_dir = dir.context();
    let context =
        context_dir
            .election_event_context_payload()
            .map_err(|e| RunnerError::FileStructure {
                msg: "election_event_context_payload".to_string(),
                source: Box::new(e),
            })?;
    let _ = rust_ev_system_library::rust_ev_crypto_primitives::prelude::prepare_fixed_based_optimization(
        context.encryption_group.g(),
        context.encryption_group.p(),
    );
    Ok(())
}
