//! Module implementing all the verifications

pub mod error;
pub mod meta_data;
pub mod setup;
pub mod tally;
pub mod verification;
pub mod verification_suite;

use self::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    constants::direct_trust_path,
    crypto_primitives::{hashing::HashableMessage, signature::VerifiySignatureTrait},
    error::{create_result_with_error, create_verifier_error, VerifierError},
};
use std::fmt::Display;

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

/// Verify the signatue for a given object implementing [VerifiySignatureTrait]
fn verify_signature_for_object<'a, T>(obj: &'a T, result: &mut VerificationResult, name: &str)
where
    T: VerifiySignatureTrait<'a>,
    HashableMessage<'a>: From<&'a T>,
{
    match obj.verifiy_signature(&direct_trust_path()) {
        Ok(t) => {
            if !t {
                result.push_failure(create_verification_failure!(format!(
                    "Wrong signature for {}",
                    name
                )))
            }
        }
        Err(e) => {
            result.push_error(create_verification_error!(
                format!("Error testing signature of {}", name),
                e
            ));
        }
    }
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
