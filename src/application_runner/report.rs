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

type VecTitleKeyValueStringType = Vec<(String, Vec<(String, String)>)>;

/// Trait to collect the report information
pub trait ReportInformationTrait {
    /// Transform the report information to a readable structure containing a
    /// grouping of information.
    ///
    /// The strucutre is a [Vec] of tuple:
    /// - The first element is the name of the group
    /// - The second element is the information in the group in for of a [Vec] of tuples
    ///     - The first element of the nested tuple is the name of the information
    ///     - The second element of the nested tuple is the information as [String]
    fn to_title_key_value(&self) -> Result<VecTitleKeyValueStringType, ReportError>;

    /// Transform the information to a multiline string.
    ///
    /// Take the verifier configuration as input for the tab size
    fn info_to_string(&self, config: &'static VerifierConfig) -> Result<String, ReportError> {
        Ok(self
            .to_title_key_value()?
            .iter()
            .map(|(title, list)| {
                let max_key_length = list.iter().map(|(k, _)| k.len()).max().unwrap();
                once(title.clone())
                    .chain(list.iter().map(|(k, s)| {
                        format!(
                            "{}{}:{} {}",
                            " ".repeat(config.txt_report_tab_size() as usize),
                            k,
                            " ".repeat(max_key_length - k.len()),
                            s,
                        )
                    }))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n\n"))
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
    ///
    /// Inputs:
    /// - `path`: Path of the verification directory
    /// - `period`: Path of the verification directory
    /// - `manual_verifications`: The manuel verifications [ManualVerifications]
    /// - `extraction_information`: The result of the extraction of the datasets [ExtractDataSetResults]
    /// - `runner_information`: Information to the runner [RunnerInformation]
    pub fn new(run_information: &'a RunInformation) -> Self {
        Self { run_information }
    }
}

impl<D: VerificationDirectoryTrait + Clone> ReportInformationTrait for ManualVerifications<D> {
    fn to_title_key_value(&self) -> Result<VecTitleKeyValueStringType, ReportError> {
        Ok(vec![
            (
                "Fingerprints".to_string(),
                self.dt_fingerprints_to_key_value(),
            ),
            ("Information".to_string(), self.information_to_key_value()),
            (
                "Verification Results".to_string(),
                self.verification_stati_to_key_value(),
            ),
        ])
    }
}

impl ReportInformationTrait for ReportData<'_> {
    fn to_title_key_value(&self) -> Result<VecTitleKeyValueStringType, ReportError> {
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
        let general_information = vec![
            ("Period".to_string(), period.to_string()),
            (
                "Context Dataset".to_string(),
                context_dataset_info
                    .source_path()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                "Context Dataset Fingerprint".to_string(),
                context_dataset_info.fingerprint_str(),
            ),
            (
                format!("{} Dataset", period),
                dataset_period_info
                    .source_path()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                format!("{} Dataset Fingerprint", period),
                dataset_period_info.fingerprint_str(),
            ),
            (
                "Verification directory".to_string(),
                self.run_information
                    .run_directory()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                "Start Time".to_string(),
                match start_time_opt {
                    Some(t) => std::convert::Into::<DateTime<Local>>::into(t)
                        .format(FORMAT_DATE_TIME)
                        .to_string(),
                    None => "Not started".to_string(),
                },
            ),
            ("Stop Time".to_string(), stop_time_str),
            (
                "Duration".to_string(),
                match duration_opt {
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
                },
            ),
        ];
        let mut res = vec![("Running information".to_string(), general_information)];
        res.append(
            &mut ManualVerifications::<VerificationDirectory>::try_from(self.run_information)?
                .to_title_key_value()?,
        );
        Ok(res)
    }
}
