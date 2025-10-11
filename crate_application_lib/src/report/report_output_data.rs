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

/// Enum with the title types
#[derive(Debug, Clone, strum::Display)]
pub enum ReportOutputDataBlockTitle {
    #[strum(to_string = "Fingerprints")]
    Fingerprints,
    #[strum(to_string = "Other fingerprints")]
    OtherFingerprints,
    #[strum(to_string = "Information")]
    Information,
    #[strum(to_string = "Verification results")]
    VerificationResults,
    #[strum(to_string = "Running Infomration")]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
}

/// Store whole Report output
#[derive(Debug, Clone)]
pub struct ReportOutputData {
    blocks: Vec<ReportOutputDataBlock>,
}

impl Default for ReportOutputData {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportOutputData {
    /// New empty report output
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    /// Report output from vec of blocks
    pub fn from_vec(blocks: Vec<ReportOutputDataBlock>) -> Self {
        Self { blocks }
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

    /// Get the blocks
    pub fn blocks(&self) -> &[ReportOutputDataBlock] {
        &self.blocks
    }
}

impl OutputToString for ReportOutputData {
    fn output_to_string(&self, tab_size: u8) -> String {
        self.blocks
            .iter()
            .map(|b| b.output_to_string(tab_size))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
