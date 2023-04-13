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

/// Marco to test the signature of a data structure
///
/// Following paramters:
/// - $fn: Name of the function to verify
/// - $d: The structure
/// - $n: The name of the structure (for the messages)
macro_rules! verifiy_signature {
    ($fn: ident, $d: ident, $n: expr) => {
        fn $fn(dir: &VerificationDirectory, result: &mut VerificationResult) {
            let setup_dir = dir.unwrap_setup();
            let eg = match setup_dir.$d() {
                Ok(p) => p,
                Err(e) => {
                    result.push_error(create_verification_error!(
                        format!("{} cannot be read", $n),
                        e
                    ));
                    return;
                }
            };
            match eg.as_ref().verifiy_signature(&direct_trust_path()) {
                Ok(t) => {
                    if !t {
                        result.push_failure(create_verification_failure!(format!(
                            "Wrong signature for {}",
                            $n
                        )))
                    }
                }
                Err(e) => {
                    result.push_error(create_verification_error!(
                        format!("Error testing signature of {}", $n),
                        e
                    ));
                }
            }
        }
    };
}
pub(crate) use verifiy_signature;
