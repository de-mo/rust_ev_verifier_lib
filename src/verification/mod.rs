//! Module implementing all the verifications

use std::fmt::Display;

use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

pub mod error;
pub mod meta_data;
pub mod setup;
pub mod tally;
pub mod verification;
pub mod verification_suite;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VerificationCategory {
    Authenticity,
    Consistency,
    Completness,
    Integrity,
    Evidence,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    Stopped,
    Running,
    Finished,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VerificationPeriod {
    Setup,
    Tally,
}

impl TryFrom<&str> for VerificationPeriod {
    type Error = VerificationPreparationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "setup" => Ok(VerificationPeriod::Setup),
            "tally" => Ok(VerificationPeriod::Tally),
            _ => create_result_with_error!(
                VerificationPreparationErrorType::VerificationPeriod,
                format!("Cannot read period from '{}'", value)
            ),
        }
    }
}

impl TryFrom<&String> for VerificationPeriod {
    type Error = VerificationPreparationError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for VerificationCategory {
    type Error = VerificationPreparationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "authenticity" => Ok(VerificationCategory::Authenticity),
            "completness" => Ok(VerificationCategory::Completness),
            "consistency" => Ok(VerificationCategory::Consistency),
            "integrity" => Ok(VerificationCategory::Integrity),
            "evidence" => Ok(VerificationCategory::Evidence),
            _ => create_result_with_error!(
                VerificationPreparationErrorType::VerificationPeriod,
                format!("Cannot read category from '{}'", value)
            ),
        }
    }
}

impl TryFrom<&String> for VerificationCategory {
    type Error = VerificationPreparationError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationPreparationErrorType {
    Metadata,
    VerificationPeriod,
    VerificationCategory,
}

impl Display for VerificationPreparationErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VerificationPreparationErrorType::Metadata => "Meta data",
            VerificationPreparationErrorType::VerificationPeriod => "Verification period",
            VerificationPreparationErrorType::VerificationCategory => "Verification category",
        };
        write!(f, "{s}")
    }
}

pub type VerificationPreparationError = VerifierError<VerificationPreparationErrorType>;
