//! Module to implement the metadata of the tests
//!
//! The metadata list is loaded from the file in resources.

use super::{VerificationCategory, VerificationPeriod};
use crate::constants::verification_list_path;
use anyhow::anyhow;
use serde::{
    de::{Deserialize as Deserialize2, Deserializer, Error},
    Deserialize,
};
use std::fs;

/// List of Verification Metadata
pub type VerificationMetaDataList = Vec<VerificationMetaData>;

/// Trait implementing functions for [VerificationMetaDataList]
///
/// Used so because it is a type of Vec.
pub trait VerificationMetaDataListTrait: Sized {
    /// Load the list from the file in resources
    fn load() -> anyhow::Result<Self>;

    // Get meta_data for id.
    fn meta_data_from_id(&self, id: &str) -> Option<&VerificationMetaData>;

    // Get the list of ids
    fn id_list(&self) -> Vec<String>;

    // Get the list of ids for the given period
    fn id_list_for_period(&self, period: &VerificationPeriod) -> Vec<String>;
}

/// Metadata of a verification
#[derive(Deserialize, Debug, Clone)]
pub struct VerificationMetaData {
    /// id of the verification
    pub id: String,

    /// Name of the verification
    pub name: String,

    /// Algorithm in the specifications
    pub algorithm: String,

    /// Description of the verification
    pub description: String,

    /// Period (Set or Tally) of the verification
    #[serde(deserialize_with = "deserialize_string_to_period")]
    pub period: VerificationPeriod,

    /// Category of the verification
    #[serde(deserialize_with = "deserialize_string_to_category")]
    pub category: VerificationCategory,
}

impl VerificationMetaDataListTrait for VerificationMetaDataList {
    fn load() -> anyhow::Result<Self> {
        let path = verification_list_path();
        let s = fs::read_to_string(&path).map_err(|e| {
            anyhow!(e).context(format!("Cannot read file {}", path.to_str().unwrap()))
        })?;
        serde_json::from_str(&s).map_err(|e| {
            anyhow!(e).context(format!(
                "Cannot deserialize json for file {}",
                path.to_str().unwrap()
            ))
        })
    }

    fn meta_data_from_id(&self, id: &str) -> Option<&VerificationMetaData> {
        self.iter().find(|e| e.id == id)
    }

    fn id_list(&self) -> Vec<String> {
        self.iter().map(|e| e.id.clone()).collect::<Vec<String>>()
    }

    fn id_list_for_period(&self, period: &VerificationPeriod) -> Vec<String> {
        self.iter()
            .filter(|e| &e.period == period)
            .map(|e| e.id.clone())
            .collect::<Vec<String>>()
    }
}

fn deserialize_string_to_period<'de, D>(deserializer: D) -> Result<VerificationPeriod, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    VerificationPeriod::try_from(&buf).map_err(|e| Error::custom(e.to_string()))
}

fn deserialize_string_to_category<'de, D>(deserializer: D) -> Result<VerificationCategory, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    VerificationCategory::try_from(&buf).map_err(|e| Error::custom(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load() {
        let metadata_res = VerificationMetaDataList::load();
        assert!(metadata_res.is_ok());
        let metadata = metadata_res.unwrap();
        assert!(metadata.len() > 0);
        assert!(metadata.meta_data_from_id("01.01").is_some())
    }
}
