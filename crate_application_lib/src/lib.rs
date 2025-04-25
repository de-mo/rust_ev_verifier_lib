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

//! Crate implementing common functionalities for all Verifier applications (console and GUI)
//!
//! Following functionalities are provided
//! - [runner::Runner] provides the possibility to run all the verifications
//! - `extract` provides the functionalities to extract the zip files
//! - [run_information::RunInformation] stores all the information about the current running
//! - [report] provides the possibility to report the actual stituation

mod extract;
pub mod report;
mod run_information;
mod runner;

pub use extract::*;
//pub use report::*;
pub use run_information::RunInformation;
pub use runner::{
    no_action_after_fn, no_action_after_runner_fn, no_action_before_fn, no_action_before_runner_fn,
    RunParallel, Runner, RunnerInformation, VerificationRunInformation,
};
use rust_ev_verifier_lib::{
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
    #[error("Error for RunInformation: {0}")]
    RunInformationError(String),
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
