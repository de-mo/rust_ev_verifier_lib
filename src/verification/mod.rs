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
    result::{VerificationEvent, VerificationResult},
    suite::VerificationSuite,
};

use crate::{
    config::Config,
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
pub enum VerificationStatus {
    Stopped,
    Running,
    Finished,
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

// Enum representing the verification errors
#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Error parsing json {msg} -> caused by: {source}")]
    ParseJSON {
        msg: String,
        source: serde_json::Error,
    },
    #[error(transparent)]
    DirectTrust(DirectTrustError),
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
    Generic(String),
}

impl VerificationPeriod {
    pub fn is_setup(&self) -> bool {
        self == &VerificationPeriod::Setup
    }

    pub fn is_tally(&self) -> bool {
        self == &VerificationPeriod::Tally
    }
}

pub(super) fn verification_unimplemented<D: VerificationDirectoryTrait>(
    _dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    result.push(VerificationEvent::new_error(
        "Verification is not implemented",
    ));
}

/// Verify the signatue for a given object implementing [VerifiySignatureTrait]
fn verify_signature_for_object<'a, T>(obj: &'a T, config: &'static Config) -> VerificationResult
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
