use crate::VerifierConfig;
use super::ReportError;
use std::iter::once;


/// Trait to transform the outputs to string
pub trait OutputToString {
    /// Transform the output to a multiline string.
    ///
    /// Take the verifier configuration as input for the tab size
    fn output_to_string(&self, config: &'static VerifierConfig) -> Result<String, ReportError>;
}

/// Structure to store an output entry
#[derive(Debug, Clone)]
pub enum ReportOutputEntry {
    /// A tuple key, value
    KeyValue((String, String)),
    /// Only a value
    OnlyValue(String),
}

/// Structure to store a block for output
#[derive(Debug, Clone)]
pub struct ReportOutputBlock {
    /// Title of the block
    title: String,
    /// Entries of the block
    entries: Vec<ReportOutputEntry>,
}

impl OutputToString for ReportOutputBlock {
    fn output_to_string(&self, config: &'static VerifierConfig) -> Result<String, ReportError> {
        let max_key_length = self.max_key_length();
        Ok(once(self.title.clone())
            .chain(self.entries.iter().map(|e| match e {
                ReportOutputEntry::KeyValue((k, v)) => format!(
                    "{}{}:{} {}",
                    " ".repeat(config.txt_report_tab_size() as usize),
                    k,
                    " ".repeat(max_key_length - k.len()),
                    v,
                ),
                ReportOutputEntry::OnlyValue(v) => v.clone(),
            }))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

impl From<&str> for ReportOutputEntry {
    fn from(value: &str) -> Self {
        Self::OnlyValue(value.to_string())
    }
}

impl From<(&str, &str)> for ReportOutputEntry {
    fn from(value: (&str, &str)) -> Self {
        Self::KeyValue((value.0.to_string(), value.1.to_string()))
    }
}

impl ReportOutputBlock {
    /// New empty block
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            entries: vec![],
        }
    }

    /// New block with entries
    pub fn new_with_entries(title: &str, entries: Vec<ReportOutputEntry>) -> Self {
        Self {
            title: title.to_string(),
            entries,
        }
    }

    /// New block with tuples
    pub fn new_with_tuples(title: &str, entries: &[(String, String)]) -> Self {
        Self::new_with_entries(
            title,
            entries
                .iter()
                .map(|(k, v)| ReportOutputEntry::from((k.as_str(), v.as_str())))
                .collect(),
        )
    }

    /// Push an entry
    pub fn push(&mut self, entry: ReportOutputEntry) {
        self.entries.push(entry);
    }

    /// Calculate the max length of the keys of the entries
    ///
    /// Return 0 if no entry has key, or with there is no entry
    pub fn max_key_length(&self) -> usize {
        self.entries
            .iter()
            .filter_map(|e| match e {
                ReportOutputEntry::KeyValue((k, _)) => Some(k.len()),
                ReportOutputEntry::OnlyValue(_) => None,
            })
            .max()
            .unwrap_or(0usize)
    }
}

/// Store whole Report output
#[derive(Debug, Clone)]
pub struct ReportOutput {
    blocks: Vec<ReportOutputBlock>,
}

impl Default for ReportOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportOutput {
    /// New empty report output
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    /// Report output from vec of blocks
    pub fn from_vec(blocks: Vec<ReportOutputBlock>) -> Self {
        Self { blocks }
    }

    /// Push a block
    pub fn push(&mut self, element: ReportOutputBlock) {
        self.blocks.push(element);
    }

    /// Append an other [ReportOutput].
    ///
    /// `other` is emptied
    pub fn append(&mut self, other: &mut Self) {
        self.blocks.append(&mut other.blocks);
    }
}

impl OutputToString for ReportOutput {
    fn output_to_string(&self, config: &'static VerifierConfig) -> Result<String, ReportError> {
        Ok(self
            .blocks
            .iter()
            .map(|b| b.output_to_string(config))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n\n"))
    }
}
