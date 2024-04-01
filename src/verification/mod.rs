//! Module implementing all the verifications

pub mod meta_data;
pub mod result;
mod setup;
pub mod suite;
mod tally;
pub mod verifications;

use self::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config, direct_trust::VerifiySignatureTrait, file_structure::VerificationDirectoryTrait,
};
use anyhow::{anyhow, bail, Result};
use log::debug;
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

impl VerificationPeriod {
    #[allow(dead_code)]
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
    result.push(create_verification_error!(anyhow!(
        "Verification is not implemented"
    )));
}

/// Verify the signatue for a given object implementing [VerifiySignatureTrait]
fn verify_signature_for_object<'a, T>(
    obj: &'a T,
    result: &mut VerificationResult,
    config: &'static Config,
    name: &str,
) where
    T: VerifiySignatureTrait<'a>,
{
    let ks = match config.keystore() {
        Ok(ks) => ks,
        Err(e) => {
            result.push(create_verification_error!("Cannot read keystore", e));
            return;
        }
    };
    let res = obj.verify_signatures(&ks);
    for (i, r) in res.iter().enumerate() {
        match r {
            Ok(t) => {
                if !t {
                    result.push(create_verification_failure!(format!(
                        "Wrong signature for {}",
                        name
                    )))
                }
            }
            Err(e) => {
                result.push(create_verification_error!(format!(
                    "Error testing signature of {} at position {}: {}",
                    name,
                    i,
                    e.to_string()
                )));
            }
        }
    }

    /*  {
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
    }*/
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

impl Display for VerificationPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationPeriod::Setup => write!(f, "setup"),
            VerificationPeriod::Tally => write!(f, "tally"),
        }
    }
}

impl Display for VerificationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationCategory::Authenticity => write!(f, "authenticity"),
            VerificationCategory::Consistency => write!(f, "consistency"),
            VerificationCategory::Completness => write!(f, "completness"),
            VerificationCategory::Integrity => write!(f, "integrity"),
            VerificationCategory::Evidence => write!(f, "evidence"),
        }
    }
}
