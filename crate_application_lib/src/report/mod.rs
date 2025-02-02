mod report_config;
mod report_output;

pub use report_config::{ReportConfig, ReportConfigBuilder};
pub use report_output::ReportOutput;

use super::{run_information::RunInformation, RunnerError};
use chrono::{DateTime, Local};
use report_output::{OutputToString, ReportOutputBlock, ReportOutputBlockTitle, ReportOutputEntry};
use rust_ev_verifier_lib::{
    file_structure::{VerificationDirectory, VerificationDirectoryTrait},
    verification::{ManualVerificationInformationTrait, ManualVerifications, VerificationPeriod},
    DatasetTypeKind,
};
use std::fmt::Display;
use thiserror::Error;
use tracing::{debug, error, info, trace, warn, Level};

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

/// Trait to collect the report information
pub trait ReportInformationTrait {
    /// Transform the report information to a [ReportOutput].
    fn to_report_output(&self) -> Result<ReportOutput, ReportError>;

    /// Transform the information to a multiline string.
    ///
    /// Take the verifier configuration as input for the tab size
    fn info_to_string(&self, tab_size: u8) -> Result<String, ReportError> {
        self.to_report_output()?.output_to_string(tab_size)
    }
}

/// Structure containing the data of the report
pub struct ReportData<'a> {
    report_configuration: ReportConfig,
    run_information: &'a RunInformation,
}

impl Display for ReportData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.info_to_string(*self.report_configuration.tab_size()) {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "ERROR generating text of report {}", e),
        }
    }
}

impl<'a> ReportData<'a> {
    /// Create new [ReportData]
    pub fn new(report_configuration: ReportConfig, run_information: &'a RunInformation) -> Self {
        let res = Self {
            report_configuration,
            run_information,
        };
        if *res.report_configuration.output_log() {
            res.trace();
        }
        res
    }

    /// Trace the [ReportData] according to the configuration
    pub fn trace(&self) {
        let s = self.to_string();
        match *self.report_configuration.output_log_level() {
            Level::TRACE => trace!("Report: \n{}", s),
            Level::DEBUG => debug!("Report: \n{}", s),
            Level::INFO => info!("Report: \n{}", s),
            Level::WARN => warn!("Report: \n{}", s),
            Level::ERROR => error!("Report: \n{}", s),
        }
    }
}

impl<D: VerificationDirectoryTrait> ReportInformationTrait for ManualVerifications<D> {
    fn to_report_output(&self) -> Result<ReportOutput, ReportError> {
        Ok(ReportOutput::from_vec(vec![
            ReportOutputBlock::new_with_tuples(
                ReportOutputBlockTitle::Fingerprints,
                &self.dt_fingerprints_to_key_value(),
            ),
            ReportOutputBlock::new_with_tuples(
                ReportOutputBlockTitle::Information,
                &self.information_to_key_value(),
            ),
            ReportOutputBlock::new_with_tuples(
                ReportOutputBlockTitle::VerificationResults,
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
            .format(self.report_configuration.fromat_date())
            .to_string(),
            false => "Not finished".to_string(),
        };
        let mut running_information =
            ReportOutputBlock::new(ReportOutputBlockTitle::RunningInformation);
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
                .format(self.report_configuration.fromat_date())
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
