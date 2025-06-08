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

//! Module implementing all the verifications

mod manual;
mod meta_data;
mod result;
mod setup;
mod suite;
mod tally;
mod verifications;

use std::fmt::Display;

pub use self::{
    manual::*,
    meta_data::*,
    result::{VerficationsWithErrorAndFailuresType, VerificationEvent, VerificationResult},
    setup::get_verifications as get_verifications_setup,
    suite::VerificationSuite,
    tally::get_verifications as get_verifications_tally,
};
use crate::{
    config::{VerifierConfig, VerifierConfigError},
    data_structures::DataStructureError,
    direct_trust::{DirectTrustError, VerifiySignatureTrait},
    file_structure::{FileStructureError, VerificationDirectoryTrait},
};
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum VerificationCategory {
    Authenticity,
    Consistency,
    Completness,
    Integrity,
    Evidence,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "lowercase")]
///  Status of a verification
pub enum VerificationStatus {
    /// Verification not started
    #[strum(serialize = "Not started")]
    NotStarted,
    /// Verification is running
    #[strum(serialize = "Running")]
    Running,
    /// Verification finished without error or failure
    #[strum(serialize = "Successful")]
    FinishedSuccessfully,
    /// Verification finished only with failures
    #[strum(serialize = "Failures")]
    FinishedWithFailures,
    /// Verification finished only with errors
    #[strum(serialize = "Errors")]
    FinishedWithErrors,
    /// Verification finished only with errors and failures
    #[strum(serialize = "Failures and Errors")]
    FinishedWithFailuresAndErrors,
}

impl VerificationStatus {
    /// For the finished verification, calculate the finished status
    /// according to the fact that the verification has errors and/or has failures
    pub fn calculate_finished(has_errors: bool, has_failures: bool) -> Self {
        match has_errors {
            true => match has_failures {
                true => Self::FinishedWithFailuresAndErrors,
                false => Self::FinishedWithErrors,
            },
            false => match has_failures {
                true => Self::FinishedWithFailures,
                false => Self::FinishedSuccessfully,
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum VerificationPeriod {
    Setup,
    Tally,
}

impl Display for VerificationPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VerificationPeriod::Setup => "Setup",
                VerificationPeriod::Tally => "Tally",
            }
        )
    }
}

#[derive(Error, Debug)]
#[error(transparent)]
/// Error during the verification process
pub struct VerificationError(#[from] VerificationErrorImpl);

#[derive(Error, Debug)]
enum VerificationErrorImpl {
    #[error("Error loading metadata")]
    LoadMetadata { source: serde_json::Error },
    #[error("Error loading metadata for period {period}")]
    LoadMetadataPeriod {
        period: VerificationPeriod,
        source: Box<VerificationError>,
    },
    #[error("Error getting the keystore creating the manual verifications for all periods")]
    KeystoreNewAll { source: VerifierConfigError },
    #[error("Error getting the fingerprints of the certificate creating the manual verifications for all periods")]
    FingerprintsNewAll { source: DirectTrustError },
    #[error("Error getting the election event context creating the manual verifications for all periods")]
    EEContextNewAll { source: Box<FileStructureError> },
    #[error(
        "Error getting the election event context creating the manual verifications for tally"
    )]
    EEContextNewTally { source: Box<FileStructureError> },
    #[error("Ballot box {bb_id} not found in the directories")]
    BBNotFoundNewTally { bb_id: String },
    #[error(
        "Error reading {bb_id}/tally_component_votes_payload creating the manual verifications for tally"
    )]
    BBVotesNewTally {
        bb_id: String,
        source: Box<FileStructureError>,
    },
    #[error(
        "Error creating the manual verifications for all period creating the manual verifications for tally"
    )]
    NewAllInNewTally { source: Box<VerificationError> },
    #[error(
        "Error creating the manual verifications for all period creating the manual verifications for setup"
    )]
    NewAllInNewSetup { source: Box<VerificationError> },
    #[error("Error collecting metadata creating the manual verifications")]
    MetadataNew { source: Box<VerificationError> },
    #[error("Error creating manual verifications for {period} creating the manual verifications")]
    NewManual {
        period: VerificationPeriod,
        source: Box<VerificationError>,
    },
    #[error("Error creating the inputs for the manual verifications from the election event context (creating verifications for all periods)")]
    VerifInputsNewAll { source: DataStructureError },
    #[error("Metadata for verification {id} not found in the list of metadata (creating the verification")]
    MetadataNotFound { id: String },
    #[error("name {name} for verification id {id} doesn't match with give name {input_name} (creating the verification)")]
    NameMismatch {
        name: String,
        id: String,
        input_name: String,
    },
    /*
    #[error("Error parsing json {msg} -> caused by: {source}")]
    ParseJSON {
        msg: String,
        source: serde_json::Error,
    },
    #[error(transparent)]
    DirectTrust(DirectTrustError),
    #[error(transparent)]
    ConfigError(VerifierConfigError),
    #[error(transparent)]
    DataStructure(DataStructureError),
    #[error("metadata for verification id {0} not found")]
    MetadataNotFound(String),
    #[error("{msg} -> caused by: {source}")]
    FileStructureError {
        msg: String,
        source: Box<FileStructureError>,
    },
    #[error("{0}")]
    Generic(String), */
}

impl VerificationPeriod {
    /// Is the period Setup
    pub fn is_setup(&self) -> bool {
        self == &VerificationPeriod::Setup
    }

    /// Is the period Tally
    pub fn is_tally(&self) -> bool {
        self == &VerificationPeriod::Tally
    }
}

pub(super) fn verification_unimplemented<D: VerificationDirectoryTrait>(
    _dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    result.push(VerificationEvent::new_error(
        "Verification is not implemented",
    ));
}

/// Verify the signatue for a given object implementing [VerifiySignatureTrait]
fn verify_signature_for_object<'a, T>(
    obj: &'a T,
    config: &'static VerifierConfig,
) -> VerificationResult
where
    T: VerifiySignatureTrait<'a>,
{
    let mut result = VerificationResult::new();
    let ks = match config.keystore() {
        Ok(ks) => ks,
        Err(e) => {
            result.push(VerificationEvent::new_error(&e).add_context("Cannot read keystore"));
            return result;
        }
    };
    let res = obj.verify_signatures(&ks);
    for (i, r) in res.iter().enumerate() {
        match r {
            Ok(t) => {
                if !t {
                    result.push(VerificationEvent::new_failure("Wrong signature"))
                }
            }
            Err(e) => {
                result.push(
                    VerificationEvent::new_failure(e).add_context(format!("at position {}", i)),
                );
            }
        }
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_finished() {
        assert_eq!(
            VerificationStatus::calculate_finished(false, false),
            VerificationStatus::FinishedSuccessfully
        );
        assert_eq!(
            VerificationStatus::calculate_finished(true, false),
            VerificationStatus::FinishedWithErrors
        );
        assert_eq!(
            VerificationStatus::calculate_finished(false, true),
            VerificationStatus::FinishedWithFailures
        );
        assert_eq!(
            VerificationStatus::calculate_finished(true, true),
            VerificationStatus::FinishedWithFailuresAndErrors
        );
    }
}
