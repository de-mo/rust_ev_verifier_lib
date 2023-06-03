//! Module implementing all the verifications

pub mod meta_data;
pub mod result;
pub mod setup;
pub mod suite;
pub mod tally;
pub mod verification;

use self::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    constants::direct_trust_path,
    crypto_primitives::{hashing::HashableMessage, signature::VerifiySignatureTrait},
};
use anyhow::{anyhow, bail};
use log::debug;

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
                result.push(create_verification_failure!(format!(
                    "Wrong signature for {}",
                    name
                )))
            }
        }
        Err(e) => {
            result.push(create_verification_error!(
                format!("Error testing signature of {}", name),
                e
            ));
        }
    }
}

impl TryFrom<&str> for VerificationPeriod {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "setup" => Ok(VerificationPeriod::Setup),
            "tally" => Ok(VerificationPeriod::Tally),
            _ => bail!(format!("Cannot read period from value '{}'", value)),
        }
    }
}

impl TryFrom<&String> for VerificationPeriod {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for VerificationCategory {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "authenticity" => Ok(VerificationCategory::Authenticity),
            "completness" => Ok(VerificationCategory::Completness),
            "consistency" => Ok(VerificationCategory::Consistency),
            "integrity" => Ok(VerificationCategory::Integrity),
            "evidence" => Ok(VerificationCategory::Evidence),
            _ => bail!(format!("Cannot category period from value '{}'", value)),
        }
    }
}

impl TryFrom<&String> for VerificationCategory {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
