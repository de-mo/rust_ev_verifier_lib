mod v1002_verify_tally_control_component;

use super::super::{suite::VerificationList, verifications::Verification};
use crate::{
    config::Config,
    verification::{
        meta_data::VerificationMetaDataList, verification_unimplemented, VerificationError,
    },
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> Result<VerificationList<'a>, VerificationError> {
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
            v1002_verify_tally_control_component::fn_verification,
            metadata_list,
            config,
        )?,
    ]))
}
