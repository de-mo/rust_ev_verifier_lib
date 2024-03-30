use super::super::{suite::VerificationList, verifications::Verification};
use crate::{
    config::Config,
    verification::{meta_data::VerificationMetaDataList, verification_unimplemented},
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> anyhow::Result<VerificationList<'a>> {
    Ok(VerificationList(vec![
        Verification::new(
            "10.01",
            "VerifyOnlineControlComponents",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "10.02",
            "VerifyTallyControlComponent",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
    ]))
}
