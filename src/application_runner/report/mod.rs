use super::{run_information::RunInformation, RunnerError};
use crate::{
    data_structures::dataset::DatasetTypeKind,
    file_structure::{VerificationDirectory, VerificationDirectoryTrait},
    verification::{ManualVerificationInformationTrait, ManualVerifications, VerificationPeriod},
    VerifierConfig,
};
use chrono::{DateTime, Local};
use std::fmt::Display;
use std::iter::once;
use thiserror::Error;

const FORMAT_DATE_TIME: &str = "%d.%m.%Y %H:%M:%S.%3f";

// Enum representing the datza structure errors
#[derive(Error, Debug)]
pub enum ReportError {
    #[error("IO error {msg} -> caused by: {source}")]
    IO { msg: String, source: std::io::Error },
    #[error("Error transforming to title, key, value: {0}")]
    ToTitleKeyValue(String),
    #[error(transparent)]
    RunnerError(#[from] RunnerError),
}

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

/// Trait to collect the report information
pub trait ReportInformationTrait {
    /// Transform the report information to a [ReportOutput].
    fn to_report_output(&self) -> Result<ReportOutput, ReportError>;

    /// Transform the information to a multiline string.
    ///
    /// Take the verifier configuration as input for the tab size
    fn info_to_string(&self, config: &'static VerifierConfig) -> Result<String, ReportError> {
        self.to_report_output()?.output_to_string(config)
    }
}

/// Structure containing the data of the report
pub struct ReportData<'a> {
    run_information: &'a RunInformation,
}

impl Display for ReportData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.info_to_string(self.run_information.config()) {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "ERROR generating text of report {}", e),
        }
    }
}

impl<'a> ReportData<'a> {
    /// Create new [ReportData]
    pub fn new(run_information: &'a RunInformation) -> Self {
        Self { run_information }
    }
}

impl<D: VerificationDirectoryTrait + Clone> ReportInformationTrait for ManualVerifications<D> {
    fn to_report_output(&self) -> Result<ReportOutput, ReportError> {
        Ok(ReportOutput::from_vec(vec![
            ReportOutputBlock::new_with_tuples(
                "Fingerprints",
                &self.dt_fingerprints_to_key_value(),
            ),
            ReportOutputBlock::new_with_tuples("Information", &self.information_to_key_value()),
            ReportOutputBlock::new_with_tuples(
                "Verification Results",
                &self.verification_stati_to_key_value(),
            ),
        ]))
    }
}

impl ReportInformationTrait for ReportData<'_> {
    fn to_report_output(&self) -> Result<ReportOutput, ReportError> {
        if !self.run_information.is_prepared() {
            return Err(ReportError::ToTitleKeyValue(
                "The run information used is not prepared".to_string(),
            ));
        }
        let period = self.run_information.verification_period().unwrap();
        let extracted_information = self.run_information.extracted_dataset_result().unwrap();
        let context_dataset_info = extracted_information
            .dataset_metadata(&DatasetTypeKind::Context)
            .unwrap();
        let dataset_period_info = match period {
            VerificationPeriod::Setup => extracted_information
                .dataset_metadata(&DatasetTypeKind::Setup)
                .unwrap(),
            VerificationPeriod::Tally => extracted_information
                .dataset_metadata(&DatasetTypeKind::Tally)
                .unwrap(),
        };
        let start_time_opt = self.run_information.runner_information().start_time;
        let duration_opt = self.run_information.runner_information().duration;
        let stop_time_str = match start_time_opt.is_some() && duration_opt.is_some() {
            true => std::convert::Into::<DateTime<Local>>::into(
                start_time_opt.unwrap() + duration_opt.unwrap(),
            )
            .format(FORMAT_DATE_TIME)
            .to_string(),
            false => "Not finished".to_string(),
        };
        let mut running_information = ReportOutputBlock::new("Running information");
        running_information.push(ReportOutputEntry::from(("Period", period.as_ref())));
        running_information.push(ReportOutputEntry::from((
            "Context Dataset",
            context_dataset_info.source_path().to_str().unwrap(),
        )));
        running_information.push(ReportOutputEntry::from((
            "Context Dataset Fingerprint",
            context_dataset_info.fingerprint_str().as_str(),
        )));
        running_information.push(ReportOutputEntry::from((
            format!("{} Dataset", period).as_str(),
            dataset_period_info.source_path().to_str().unwrap(),
        )));
        running_information.push(ReportOutputEntry::from((
            format!("{} Dataset Fingerprint", period).as_str(),
            dataset_period_info.fingerprint_str().as_str(),
        )));
        running_information.push(ReportOutputEntry::from((
            "Verification directory",
            self.run_information.run_directory().to_str().unwrap(),
        )));
        let start_time_string = match start_time_opt {
            Some(t) => std::convert::Into::<DateTime<Local>>::into(t)
                .format(FORMAT_DATE_TIME)
                .to_string(),
            None => "Not started".to_string(),
        };
        running_information.push(ReportOutputEntry::from((
            "Start Time",
            start_time_string.as_str(),
        )));
        running_information.push(ReportOutputEntry::from((
            "Stop Time",
            stop_time_str.as_str(),
        )));
        let duration_string = match duration_opt {
            Some(d) => {
                let mut s = d.as_secs();
                let res;
                if s < 60 {
                    res = format!("{s}s");
                } else {
                    let mut m = s / 60;
                    s %= 60;
                    if m < 60 {
                        res = format!("{m}m {s}s");
                    } else {
                        let h = m / 60;
                        m %= 60;
                        res = format!("{h}h {m}m {s}s")
                    }
                }
                res
            }
            None => "Not finished".to_string(),
        };
        running_information.push(ReportOutputEntry::from((
            "Duration",
            duration_string.as_str(),
        )));
        let mut res = ReportOutput::new();
        res.push(running_information);
        res.append(
            &mut ManualVerifications::<VerificationDirectory>::try_from(self.run_information)?
                .to_report_output()?,
        );
        Ok(res)
    }
}
