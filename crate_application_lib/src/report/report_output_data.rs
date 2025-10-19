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

use std::iter::once;

use derive_builder::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// Enum with the title types
#[derive(Debug, Clone, strum::Display, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportOutputDataBlockTitle {
    #[strum(to_string = "Fingerprints")]
    Fingerprints,
    #[strum(to_string = "Other fingerprints")]
    OtherFingerprints,
    #[strum(to_string = "Information")]
    Information,
    #[strum(to_string = "Verification results")]
    VerificationResults,
    #[strum(to_string = "Running Information")]
    RunningInformation,
    #[strum(to_string = "Errors for {0}")]
    VerificationErrors(String),
    #[strum(to_string = "Failures for {0}")]
    VerificationFailures(String),
}

/// Trait to transform the outputs to string
pub trait OutputToString {
    /// Transform the output to a multiline string.
    ///
    /// Take the verifier configuration as input for the tab size
    fn output_to_string(&self, tab_size: u8) -> String;
}

/// Structure to store an output entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportOutputDataEntry {
    /// A tuple key, value
    KeyValue((String, String)),
    /// Only a value
    OnlyValue(String),
}

impl ReportOutputDataEntry {
    pub fn is_key_value(&self) -> bool {
        matches!(self, ReportOutputDataEntry::KeyValue(_))
    }

    pub fn unwrap_key_value(&self) -> (&str, &str) {
        match self {
            ReportOutputDataEntry::KeyValue((k, v)) => (k.as_str(), v.as_str()),
            ReportOutputDataEntry::OnlyValue(_) => panic!("Called unwrap_key_value on OnlyValue"),
        }
    }

    pub fn unwrap_only_value(&self) -> &str {
        match self {
            ReportOutputDataEntry::OnlyValue(v) => v.as_str(),
            ReportOutputDataEntry::KeyValue(_) => panic!("Called unwrap_only_value on KeyValue"),
        }
    }
}

/// Structure to store a block for output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportOutputDataBlock {
    /// Title of the block
    title: ReportOutputDataBlockTitle,
    /// Entries of the block
    entries: Vec<ReportOutputDataEntry>,
}

impl OutputToString for ReportOutputDataBlock {
    fn output_to_string(&self, tab_size: u8) -> String {
        let max_key_length = self.max_key_length();
        once(self.title.to_string())
            .chain(self.entries.iter().map(|e| match e {
                ReportOutputDataEntry::KeyValue((k, v)) => format!(
                    "{}{}:{} {}",
                    " ".repeat(tab_size as usize),
                    k,
                    " ".repeat(max_key_length - k.len()),
                    v,
                ),
                ReportOutputDataEntry::OnlyValue(v) => v.clone(),
            }))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl From<&str> for ReportOutputDataEntry {
    fn from(value: &str) -> Self {
        Self::OnlyValue(value.to_string())
    }
}

impl From<(&str, &str)> for ReportOutputDataEntry {
    fn from(value: (&str, &str)) -> Self {
        Self::KeyValue((value.0.to_string(), value.1.to_string()))
    }
}

impl ReportOutputDataBlock {
    /// New empty block
    pub fn new(title: ReportOutputDataBlockTitle) -> Self {
        Self {
            title,
            entries: vec![],
        }
    }

    /// New block with entries
    pub fn new_with_entries(
        title: ReportOutputDataBlockTitle,
        entries: Vec<ReportOutputDataEntry>,
    ) -> Self {
        Self { title, entries }
    }

    /// New block with tuples
    pub fn new_with_tuples(
        title: ReportOutputDataBlockTitle,
        entries: &[(String, String)],
    ) -> Self {
        Self::new_with_entries(
            title,
            entries
                .iter()
                .map(|(k, v)| ReportOutputDataEntry::from((k.as_str(), v.as_str())))
                .collect(),
        )
    }

    /// New block with tuples
    pub fn new_with_strings(title: ReportOutputDataBlockTitle, entries: &[String]) -> Self {
        Self::new_with_entries(
            title,
            entries
                .iter()
                .map(|s| ReportOutputDataEntry::from(s.as_str()))
                .collect(),
        )
    }

    /// Push an entry
    pub fn push(&mut self, entry: ReportOutputDataEntry) {
        self.entries.push(entry);
    }

    /// Calculate the max length of the keys of the entries
    ///
    /// Return 0 if no entry has key, or with there is no entry
    pub fn max_key_length(&self) -> usize {
        self.entries
            .iter()
            .filter_map(|e| match e {
                ReportOutputDataEntry::KeyValue((k, _)) => Some(k.len()),
                ReportOutputDataEntry::OnlyValue(_) => None,
            })
            .max()
            .unwrap_or(0usize)
    }

    /// Get the title
    pub fn title(&self) -> &ReportOutputDataBlockTitle {
        &self.title
    }

    /// Get the entries
    pub fn entries(&self) -> &[ReportOutputDataEntry] {
        &self.entries
    }

    /// Get all entries of type [ReportOutputDataEntry::KeyValue]
    pub fn key_value_entries(&self) -> Vec<(&str, &str)> {
        self.entries()
            .iter()
            .filter(|e| e.is_key_value())
            .map(|entry| entry.unwrap_key_value())
            .collect()
    }

    /// Get all entries of type [ReportOutputDataEntry::OnlyValue]
    pub fn only_value_entries(&self) -> Vec<&str> {
        self.entries()
            .iter()
            .filter(|e| !e.is_key_value())
            .map(|entry| entry.unwrap_only_value())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Getters, Builder)]
#[builder(setter(into))]
pub struct ReportOutputDataMetaData {
    seed: String,
    title: String,
    date_time: String,
}

/// Store whole Report output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Getters)]
pub struct ReportOutputData {
    metadata: ReportOutputDataMetaData,
    blocks: Vec<ReportOutputDataBlock>,
}

impl ReportOutputData {
    /// New empty report output
    pub fn new(metadata: ReportOutputDataMetaData) -> Self {
        Self {
            metadata,
            blocks: vec![],
        }
    }

    /// Report output from vec of blocks
    pub fn from_vec(
        metadata: ReportOutputDataMetaData,
        blocks: Vec<ReportOutputDataBlock>,
    ) -> Self {
        Self { metadata, blocks }
    }

    /// Push a block
    pub fn push(&mut self, element: ReportOutputDataBlock) {
        self.blocks.push(element);
    }

    /// Append an other [ReportOutput].
    ///
    /// `other` is emptied
    pub fn append(&mut self, other: &mut Self) {
        self.blocks.append(&mut other.blocks);
    }
}

impl OutputToString for ReportOutputData {
    fn output_to_string(&self, tab_size: u8) -> String {
        let mut res = String::new();
        res.push_str(&format!("{}\n", self.metadata().title()));
        res.push_str(&format!("Date / Time: {}\n\n", self.metadata().date_time()));
        res.push_str(
            &self
                .blocks
                .iter()
                .map(|b| b.output_to_string(tab_size))
                .collect::<Vec<_>>()
                .join("\n\n"),
        );
        res
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::Local;

    use super::*;

    #[test]
    fn test_serialization() {
        let data = super::super::report_output_file::test::test_sample();
        let json_res = serde_json::to_string_pretty(&data);
        assert!(json_res.is_ok(), "{:?}", json_res.err());
        let json = json_res.unwrap();
        println!("Serialized JSON:\n{}", json);
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let json_path = dir.join(format!("test_report_{}.json", now));
        let mut write = std::fs::File::create(&json_path).unwrap();
        let res = serde_json::to_writer_pretty(&mut write, &data);
        assert!(res.is_ok(), "{:?}", res.err());
    }

    #[test]
    fn test_deserialization() {
        let data = super::super::report_output_file::test::test_sample();
        let json_res = serde_json::to_string_pretty(&data);
        assert!(json_res.is_ok(), "{:?}", json_res.err());
        let json = json_res.unwrap();
        let deserialized = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "{:?}", deserialized.err());
        let deserialized_data: ReportOutputData = deserialized.unwrap();
        assert_eq!(data, deserialized_data);
    }

    #[test]
    fn test_deserialization_file() {
        let data = super::super::report_output_file::test::test_sample();
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let json_path = dir.join(format!("test_report_{}.json", now));
        {
            let mut write = std::fs::File::create(&json_path).unwrap();
            serde_json::to_writer_pretty(&mut write, &data).unwrap();
        }
        let read = std::fs::File::open(&json_path).unwrap();
        let deserialized: serde_json::Result<ReportOutputData> = serde_json::from_reader(read);
        assert!(deserialized.is_ok(), "{:?}", deserialized.err());
        let deserialized_data: ReportOutputData = deserialized.unwrap();
        assert_eq!(data, deserialized_data);
    }
}
