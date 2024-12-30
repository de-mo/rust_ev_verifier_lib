use super::ExtractDataSetResults;
use crate::{
    file_structure::VerificationDirectoryTrait,
    verification::{
        ManualVerificationInformationTrait, ManualVerificationsAllPeriod, VerificationPeriod,
        VerificationSuite,
    },
};

struct ReportData<'a, D>
where
    D: VerificationDirectoryTrait,
{
    period: &'a VerificationPeriod,
    verification_suite: &'a VerificationSuite<'a>,
    manual_verifications: &'a ManualVerificationsAllPeriod<'a, D>,
    extraction_information: &'a ExtractDataSetResults,
}

impl<'a, D> ReportData<'a, D>
where
    D: VerificationDirectoryTrait,
{
    pub fn new(
        period: &'a VerificationPeriod,
        verification_suite: &'a VerificationSuite<'a>,
        manual_verifications: &'a ManualVerificationsAllPeriod<'a, D>,
        extraction_information: &'a ExtractDataSetResults,
    ) -> Self {
        Self {
            period,
            verification_suite,
            manual_verifications,
            extraction_information,
        }
    }
}
