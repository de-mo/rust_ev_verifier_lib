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

mod report_output_data;
mod report_output_file;

use crate::canonicalize_path_os_dependent;

use super::{RunnerError, run_information::RunInformation};
use derive_builder::Builder;
use derive_getters::Getters;
use report_output_data::{
    OutputToString, ReportOutputDataBlock, ReportOutputDataBlockTitle, ReportOutputDataEntry,
    ReportOutputDataMetaDataBuilderError,
};
pub use report_output_data::{
    ReportOutputData, ReportOutputDataMetaData, ReportOutputDataMetaDataBuilder,
};
use report_output_file::ReportOutputFile;
pub use report_output_file::{
    PDFReportOptions, PDFReportOptionsBuilder, ReportOutputFileOptions,
    ReportOutputFileOptionsBuilder, ReportOutputFileType,
};
use rust_ev_verifier_lib::{
    DatasetTypeKind, VerifierConfigError,
    file_structure::{VerificationDirectory, VerificationDirectoryTrait},
    verification::{ManualVerificationInformationTrait, ManualVerifications, VerificationPeriod},
};
use std::{fmt::Display, path::PathBuf};
use thiserror::Error;
use tracing::{Level, debug, error, info, trace, warn};

#[derive(Error, Debug)]
#[error(transparent)]
/// Error related to the report
pub struct ReportError(#[from] ReportErrorImpl);

#[derive(Error, Debug)]
enum ReportErrorImpl {
    #[error("Error transforming to title, key, value: {0}")]
    ToTitleKeyValue(&'static str),
    #[error("Error transforming output to string")]
    OutputToString { source: Box<ReportError> },
    #[error("Error getting the output")]
    ToOutput { source: Box<ReportError> },
    #[error("Error getting the manual verifications from the inputs")]
    Manual { source: Box<RunnerError> },
    #[error("Error with the report output options: {0}")]
    ReportOutputOptions(String),
    #[error("IO Error: {msg}")]
    IOError { msg: String, source: std::io::Error },
    #[error("Chrome headless error: {msg} \n Error: {error}")]
    Browser { msg: String, error: String },
    #[error("Error generating the report file {path}")]
    ReportError {
        path: PathBuf,
        source: Box<ReportError>,
    },
    #[error("Error building the pdf report options")]
    PdfReportOptionBuilder { source: Box<ReportError> },
    #[error("Error building the report options")]
    ReportOptionBuilder { source: Box<ReportError> },
    #[error("Error exporting to json")]
    ToJson { source: serde_json::Error },
    #[error("Error generating the metadata for the report")]
    MetadataError {
        source: ReportOutputDataMetaDataBuilderError,
    },
    #[error("Error reading the verifier configuration: {msg}")]
    VerifierConfig {
        msg: String,
        source: Box<VerifierConfigError>,
    },
    #[error("PDF report generation requires a browser path, but none is set")]
    BrowserPathNone,
}

/// General Configuration of the report
#[derive(Debug, Clone, PartialEq, Builder, Getters)]
#[builder(setter(into))]
pub struct ReportConfig {
    /// Title of the report
    title: String,

    /// Date and time of the report
    date_time: String,

    /// Size of the tabulation in the output
    tab_size: u8,

    /// Level of output log
    ///
    /// Default: `[Level::INFO]`
    #[builder(default=Level::INFO)]
    output_log_level: Level,

    /// Format of the printed date
    fromat_date: String,
}

/// Trait to collect the report information
pub trait ReportInformationTrait {
    /// Transform the report information to a [ReportOutput].
    fn to_report_output(
        &self,
        metadata: ReportOutputDataMetaData,
    ) -> Result<ReportOutputData, ReportError>;

    /// Transform the information to a multiline string.
    ///
    /// Take the verifier configuration as input for the tab size
    fn info_to_string(
        &self,
        metadata: ReportOutputDataMetaData,
        tab_size: u8,
    ) -> Result<String, ReportError> {
        Ok(self
            .to_report_output(metadata)
            .map_err(|e| ReportErrorImpl::OutputToString {
                source: Box::new(e),
            })?
            .output_to_string(tab_size))
    }

    /// Generate the report files according to the specified output options
    ///
    /// Returns a vector of [ReportError] encountered during the generation
    fn generate_files(
        &self,
        metadata: ReportOutputDataMetaData,
        output_options: ReportOutputFileOptions,
    ) -> Vec<ReportError> {
        let report_output = match self.to_report_output(metadata) {
            Ok(ro) => ro,
            Err(e) => return vec![e],
        };
        ReportOutputFile::new(output_options, &report_output).generate()
    }
}

/// Structure containing the data of the report
pub struct ReportData<'a> {
    report_configuration: ReportConfig,
    run_information: &'a RunInformation,
}

impl Display for ReportData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.info_to_string(
            match self.report_output_data_meta_data() {
                Ok(md) => md,
                Err(e) => {
                    return write!(f, "ERROR generating metadata for report display: {:?}", e);
                }
            },
            *self.report_configuration.tab_size(),
        ) {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "ERROR generating text of report {}", e),
        }
    }
}

impl<'a> ReportData<'a> {
    /// Create new [ReportData]
    pub fn new(report_configuration: ReportConfig, run_information: &'a RunInformation) -> Self {
        Self {
            report_configuration,
            run_information,
        }
    }

    fn report_output_data_meta_data(&self) -> Result<ReportOutputDataMetaData, ReportErrorImpl> {
        ReportOutputDataMetaDataBuilder::default()
            .seed(
                self.run_information
                    .runner_information()
                    .seed
                    .as_deref()
                    .unwrap_or("Unknown"),
            )
            .title(self.report_configuration.title().clone())
            .date_time(self.report_configuration.date_time().clone())
            .build()
            .map_err(|e| ReportErrorImpl::MetadataError { source: e })
    }

    /// Output the [ReportData] in the log according to the configuration
    pub fn output_log(&self) {
        let s = self.to_string();
        match *self.report_configuration.output_log_level() {
            Level::TRACE => trace!("Report: \n{}", s),
            Level::DEBUG => debug!("Report: \n{}", s),
            Level::INFO => info!("Report: \n{}", s),
            Level::WARN => warn!("Report: \n{}", s),
            Level::ERROR => error!("Report: \n{}", s),
        }
    }

    /// Export to json
    pub fn to_json(&self) -> Result<String, ReportError> {
        serde_json::to_string(&self.to_report_output(self.report_output_data_meta_data()?)?)
            .map_err(|e| ReportError::from(ReportErrorImpl::ToJson { source: e }))
    }
}

pub fn generate_files_from_json(
    json_str: &str,
    output_options: ReportOutputFileOptions,
) -> Vec<ReportError> {
    let report_output: ReportOutputData = match serde_json::from_str(json_str) {
        Ok(ro) => ro,
        Err(e) => {
            return vec![ReportError::from(ReportErrorImpl::ToJson { source: e })];
        }
    };
    ReportOutputFile::new(output_options, &report_output).generate()
}

impl<D: VerificationDirectoryTrait> ReportInformationTrait for ManualVerifications<D> {
    fn to_report_output(
        &self,
        metadata: ReportOutputDataMetaData,
    ) -> Result<ReportOutputData, ReportError> {
        let mut res = vec![
            ReportOutputDataBlock::new_with_tuples(
                ReportOutputDataBlockTitle::Fingerprints,
                &self.dt_fingerprints_to_key_value(),
            ),
            ReportOutputDataBlock::new_with_tuples(
                ReportOutputDataBlockTitle::OtherFingerprints,
                &self.other_fingerprints_to_key_value(),
            ),
            ReportOutputDataBlock::new_with_tuples(
                ReportOutputDataBlockTitle::Information,
                &self.information_to_key_value(),
            ),
            ReportOutputDataBlock::new_with_tuples(
                ReportOutputDataBlockTitle::VerificationResults,
                &self.verification_stati_to_key_value(),
            ),
        ];
        res.append(
            &mut self
                .verification_errors_and_failures()
                .iter()
                .flat_map(|(id, (errors, failures))| {
                    let mut res = vec![];
                    if !errors.is_empty() {
                        res.push(ReportOutputDataBlock::new_with_strings(
                            ReportOutputDataBlockTitle::VerificationErrors(id.clone()),
                            &errors
                                .iter()
                                .enumerate()
                                .map(|(i, s)| format!("[{}] - {}", i + 1, s))
                                .collect::<Vec<_>>(),
                        ));
                    }
                    if !failures.is_empty() {
                        res.push(ReportOutputDataBlock::new_with_strings(
                            ReportOutputDataBlockTitle::VerificationFailures(id.clone()),
                            &failures
                                .iter()
                                .enumerate()
                                .map(|(i, s)| format!("[{}] - {}", i + 1, s))
                                .collect::<Vec<_>>(),
                        ));
                    }
                    res
                })
                .collect::<Vec<_>>(),
        );
        Ok(ReportOutputData::from_vec(metadata, res))
    }
}

impl ReportInformationTrait for ReportData<'_> {
    fn to_report_output(
        &self,
        metadata: ReportOutputDataMetaData,
    ) -> Result<ReportOutputData, ReportError> {
        if !self.run_information.is_prepared() {
            return Err(ReportError::from(ReportErrorImpl::ToTitleKeyValue(
                "The run information used is not prepared",
            )));
        }
        let period = self.run_information.verification_period().unwrap();
        let extracted_information = self.run_information.extracted_dataset_result().unwrap();
        let context_dataset_info = extracted_information
            .dataset_metadata(&DatasetTypeKind::Context)
            .unwrap();
        let dataset_period_info = match period {
            VerificationPeriod::Setup => None,
            VerificationPeriod::Tally => Some(
                extracted_information
                    .dataset_metadata(&DatasetTypeKind::Tally)
                    .unwrap(),
            ),
        };
        let mut running_information =
            ReportOutputDataBlock::new(ReportOutputDataBlockTitle::RunningInformation);
        running_information.push(ReportOutputDataEntry::from(("Period", period.as_ref())));
        running_information.push(ReportOutputDataEntry::from((
            "Context Dataset",
            canonicalize_path_os_dependent(context_dataset_info.source_path()).as_str(),
        )));
        running_information.push(ReportOutputDataEntry::from((
            "Context Dataset Fingerprint",
            context_dataset_info.fingerprint_str().as_str(),
        )));
        if let Some(info) = dataset_period_info {
            running_information.push(ReportOutputDataEntry::from((
                format!("{} Dataset", period).as_str(),
                canonicalize_path_os_dependent(info.source_path()).as_str(),
            )));
            running_information.push(ReportOutputDataEntry::from((
                format!("{} Dataset Fingerprint", period).as_str(),
                info.fingerprint_str().as_str(),
            )));
        };
        running_information.push(ReportOutputDataEntry::from((
            "Verification directory",
            canonicalize_path_os_dependent(self.run_information.run_directory()).as_str(),
        )));
        running_information.push(ReportOutputDataEntry::from((
            "Start Time",
            self.run_information
                .runner_information()
                .start_time_to_string()
                .unwrap_or_else(|| "Not started".to_string())
                .as_str(),
        )));
        running_information.push(ReportOutputDataEntry::from((
            "Stop Time",
            self.run_information
                .runner_information()
                .stop_time_to_string()
                .unwrap_or_else(|| "Not finished".to_string())
                .as_str(),
        )));
        let duration_string = self
            .run_information
            .runner_information()
            .duration_as_secs_to_string()
            .unwrap_or_else(|| "Not finished".to_string());
        running_information.push(ReportOutputDataEntry::from((
            "Duration",
            duration_string.as_str(),
        )));
        let mut res = ReportOutputData::new(metadata.clone());
        res.push(running_information);
        res.append(
            &mut ManualVerifications::<VerificationDirectory>::try_from(self.run_information)
                .map_err(|e| ReportErrorImpl::Manual {
                    source: Box::new(e),
                })?
                .to_report_output(metadata)
                .map_err(|e| ReportErrorImpl::ToOutput {
                    source: Box::new(e),
                })?,
        );
        Ok(res)
    }
}
