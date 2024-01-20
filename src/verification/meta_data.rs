//! Module to implement the metadata of the tests
//!
//! The metadata list is loaded from the file in resources.

use super::{VerificationCategory, VerificationPeriod};
use anyhow::anyhow;
use serde::{
    de::{Deserialize as Deserialize2, Deserializer, Error},
    Deserialize,
};

/// List of Verification Metadata
#[derive(Deserialize, Debug, Clone)]
pub struct VerificationMetaDataList(pub Vec<VerificationMetaData>);

/// Metadata of a verification
#[derive(Deserialize, Debug, Clone)]
pub struct VerificationMetaData {
    /// id of the verification
    id: String,

    /// Name of the verification
    name: String,

    /// Algorithm in the specifications
    #[allow(dead_code)]
    algorithm: String,

    /// Description of the verification
    #[allow(dead_code)]
    description: String,

    /// Period (Set or Tally) of the verification
    #[serde(deserialize_with = "deserialize_string_to_period")]
    period: VerificationPeriod,

    /// Category of the verification
    #[serde(deserialize_with = "deserialize_string_to_category")]
    category: VerificationCategory,
}

impl VerificationMetaDataList {
    pub fn load(data: &str) -> anyhow::Result<Self> {
        serde_json::from_str(data)
            .map_err(|e| anyhow!(e).context("Cannot deserialize the verification list from json"))
    }

    pub fn load_period(data: &str, period: &VerificationPeriod) -> anyhow::Result<Self> {
        Ok(Self(
            Self::load(data)?
                .0
                .iter()
                .filter(|&m| m.period() == period)
                .cloned()
                .collect::<Vec<VerificationMetaData>>(),
        ))
    }

    pub fn meta_data_from_id(&self, id: &str) -> Option<&VerificationMetaData> {
        self.0.iter().find(|e| e.id == id)
    }

    pub fn id_list(&self) -> Vec<String> {
        self.0.iter().map(|e| e.id.clone()).collect::<Vec<String>>()
    }

    #[allow(dead_code)]
    pub fn id_list_for_period(&self, period: &VerificationPeriod) -> Vec<String> {
        self.0
            .iter()
            .filter(|e| &e.period == period)
            .map(|e| e.id.clone())
            .collect::<Vec<String>>()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, id: &str) -> Option<&VerificationMetaData> {
        self.iter().find(|&e| e.id == id)
    }

    pub fn iter(&self) -> std::slice::Iter<VerificationMetaData> {
        self.0.iter()
    }
}

impl VerificationMetaData {
    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    #[allow(dead_code)]
    pub fn algorithm(&self) -> &String {
        &self.algorithm
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn period(&self) -> &VerificationPeriod {
        &self.period
    }

    pub fn category(&self) -> &VerificationCategory {
        &self.category
    }

    pub fn from_id(id: &str, data: &str) -> Option<Self> {
        match VerificationMetaDataList::load(data) {
            Ok(l) => l.get(id).cloned(),
            Err(_) => None,
        }
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
    use crate::config::test::CONFIG_TEST;

    #[test]
    fn test_load() {
        let metadata_res = VerificationMetaDataList::load(CONFIG_TEST.get_verification_list_str());
        assert!(metadata_res.is_ok());
        let metadata = metadata_res.unwrap();
        assert!(!metadata.is_empty());
        assert!(metadata.meta_data_from_id("01.01").is_some())
    }
}
