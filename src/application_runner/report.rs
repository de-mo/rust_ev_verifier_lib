use super::{runner::RunStrategy, ExtractDataSetResults, Runner};
use crate::{
    data_structures::dataset::DatasetTypeKind,
    file_structure::VerificationDirectoryTrait,
    verification::{
        ManualVerificationInformationTrait, ManualVerificationsAllPeriod, VerificationPeriod,
    },
};
use chrono::{DateTime, Local};
use std::fmt::Display;
use std::iter::once;

const TAB_SIZE: u8 = 2;

pub trait ReportInformationTrait {
    fn to_title_key_value(&self) -> Vec<(String, Vec<(String, String)>)>;
}

struct ReportData<'a, D, T>
where
    D: VerificationDirectoryTrait,
    T: RunStrategy<'a>,
{
    period: &'a VerificationPeriod,
    runner: &'a Runner<'a, T>,
    manual_verifications: &'a ManualVerificationsAllPeriod<'a, D>,
    extraction_information: &'a ExtractDataSetResults,
}

impl Display for &dyn ReportInformationTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.to_title_key_value()
                .iter()
                .map(|(title, list)| {
                    let max_key_length = list.iter().map(|(k, _)| k.len()).max().unwrap();
                    once(title.clone())
                        .chain(list.iter().map(|(k, s)| {
                            format!(
                                "{}{}:{} {}",
                                " ".repeat(TAB_SIZE as usize),
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
        )
    }
}

impl<'a, D, T> ReportData<'a, D, T>
where
    D: VerificationDirectoryTrait,
    T: RunStrategy<'a>,
{
    pub fn new(
        period: &'a VerificationPeriod,
        runner: &'a Runner<'a, T>,
        manual_verifications: &'a ManualVerificationsAllPeriod<'a, D>,
        extraction_information: &'a ExtractDataSetResults,
    ) -> Self {
        Self {
            period,
            runner,
            manual_verifications,
            extraction_information,
        }
    }
}

impl<D: VerificationDirectoryTrait> ReportInformationTrait for ManualVerificationsAllPeriod<'_, D> {
    fn to_title_key_value(&self) -> Vec<(String, Vec<(String, String)>)> {
        vec![
            ("Fingerprints".to_string(), self.fingerprints_to_key_value()),
            ("Information".to_string(), self.information_to_key_value()),
        ]
    }
}

impl<'a, D, T> ReportInformationTrait for ReportData<'a, D, T>
where
    D: VerificationDirectoryTrait,
    T: RunStrategy<'a>,
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
                self.runner
                    .verification_directory_path()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                "Start Time".to_string(),
                match self.runner.start_time() {
                    Some(t) => std::convert::Into::<DateTime<Local>>::into(t).to_string(),
                    None => "Not started".to_string(),
                },
            ),
            (
                "Stop Time".to_string(),
                match self.runner.stop_time() {
                    Some(t) => std::convert::Into::<DateTime<Local>>::into(t).to_string(),
                    None => "Not finished".to_string(),
                },
            ),
            (
                "Duration".to_string(),
                match self.runner.duration() {
                    Some(d) => {
                        let mut s = d.as_secs();
                        let mut res = String::new();
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
