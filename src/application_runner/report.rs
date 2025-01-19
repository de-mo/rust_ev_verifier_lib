use super::{runner::RunnerInformation, ExtractDataSetResults};
use crate::{
    data_structures::dataset::DatasetTypeKind,
    file_structure::VerificationDirectoryTrait,
    verification::{
        ManualVerificationInformationTrait, ManualVerifications, VerificationPeriod,
        VerificationStatus,
    },
    VerifierConfig,
};
use chrono::{DateTime, Local};
use std::iter::once;
use std::{fmt::Display, path::Path};

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
    fn to_title_key_value(&self) -> Vec<(String, Vec<(String, String)>)>;

    /// Transform the information to a multiline string.
    ///
    /// Take the verifier configuration as input for the tab size
    fn info_to_string(&self, config: &'static VerifierConfig) -> String {
        self.to_title_key_value()
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
            .join("\n\n")
    }
}

/// Srtucture to store the information of the run of a verification
pub struct VerificationRunInformation {
    /// id of the verification
    pub id: String,
    /// status of the verification
    pub status: VerificationStatus,
    /// List of failures as [String]
    pub failures: Vec<String>,
    /// List of errors as [String]
    pub errors: Vec<String>,
}

/// Structure containing the data of the report
pub struct ReportData<'a, D>
where
    D: VerificationDirectoryTrait,
{
    path: &'a Path,
    config: &'static VerifierConfig,
    period: &'a VerificationPeriod,
    manual_verifications: &'a ManualVerifications<'a, D>,
    extraction_information: &'a ExtractDataSetResults,
    runner_information: &'a RunnerInformation,
}

impl<D> Display for ReportData<'_, D>
where
    D: VerificationDirectoryTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info_to_string(self.config))
    }
}

impl<'a, D> ReportData<'a, D>
where
    D: VerificationDirectoryTrait,
{
    /// Create new [ReportData]
    ///
    /// Inputs:
    /// - `path`: Path of the verification directory
    /// - `period`: Path of the verification directory
    /// - `manual_verifications`: The manuel verifications [ManualVerifications]
    /// - `extraction_information`: The result of the extraction of the datasets [ExtractDataSetResults]
    /// - `runner_information`: Information to the runner [RunnerInformation]
    pub fn new(
        path: &'a Path,
        config: &'static VerifierConfig,
        period: &'a VerificationPeriod,
        manual_verifications: &'a ManualVerifications<'a, D>,
        extraction_information: &'a ExtractDataSetResults,
        runner_information: &'a RunnerInformation,
    ) -> Self {
        Self {
            path,
            config,
            period,
            manual_verifications,
            extraction_information,
            runner_information,
        }
    }
}

impl<D: VerificationDirectoryTrait> ReportInformationTrait for ManualVerifications<'_, D> {
    fn to_title_key_value(&self) -> Vec<(String, Vec<(String, String)>)> {
        vec![
            (
                "Fingerprints".to_string(),
                self.dt_fingerprints_to_key_value(),
            ),
            ("Information".to_string(), self.information_to_key_value()),
            (
                "Verification Results".to_string(),
                self.verification_stati_to_key_value(),
            ),
        ]
    }
}

impl<D> ReportInformationTrait for ReportData<'_, D>
where
    D: VerificationDirectoryTrait,
{
    fn to_title_key_value(&self) -> Vec<(String, Vec<(String, String)>)> {
        let context_dataset_info = self
            .extraction_information
            .dataset_metadata(&DatasetTypeKind::Context)
            .unwrap();
        let dataset_period_info = match self.period {
            VerificationPeriod::Setup => self
                .extraction_information
                .dataset_metadata(&DatasetTypeKind::Setup)
                .unwrap(),
            VerificationPeriod::Tally => self
                .extraction_information
                .dataset_metadata(&DatasetTypeKind::Tally)
                .unwrap(),
        };
        let stop_time_str = match self.runner_information.start_time.is_some()
            && self.runner_information.duration.is_some()
        {
            true => std::convert::Into::<DateTime<Local>>::into(
                self.runner_information.start_time.unwrap()
                    + self.runner_information.duration.unwrap(),
            )
            .to_string(),
            false => "Not finished".to_string(),
        };
        let general_information = vec![
            ("Period".to_string(), self.period.to_string()),
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
                format!("{} Dataset", self.period),
                dataset_period_info
                    .source_path()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                format!("{} Dataset Fingerprint", self.period),
                dataset_period_info.fingerprint_str(),
            ),
            (
                "Verification directory".to_string(),
                self.path.to_str().unwrap().to_string(),
            ),
            (
                "Start Time".to_string(),
                match self.runner_information.start_time {
                    Some(t) => std::convert::Into::<DateTime<Local>>::into(t).to_string(),
                    None => "Not started".to_string(),
                },
            ),
            ("Stop Time".to_string(), stop_time_str),
            (
                "Duration".to_string(),
                match self.runner_information.duration {
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
        res.append(&mut self.manual_verifications.to_title_key_value());
        res
    }
}
