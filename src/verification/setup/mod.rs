//! Module implementing the verifications for setup
mod authenticity;
mod completness;
mod consistency;
mod evidence;
mod integrity;

use super::{meta_data::VerificationMetaDataList, suite::VerificationList, VerificationError};
use crate::config::Config;

/// Collect the verifications of the submodules
pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> Result<VerificationList<'a>, VerificationError> {
    let mut res = VerificationList(vec![]);
    res.0
        .append(&mut authenticity::get_verifications(metadata_list, config)?.0);
    res.0
        .append(&mut completness::get_verifications(metadata_list, config)?.0);
    res.0
        .append(&mut consistency::get_verifications(metadata_list, config)?.0);
    res.0
        .append(&mut evidence::get_verifications(metadata_list, config)?.0);
    res.0
        .append(&mut integrity::get_verifications(metadata_list, config)?.0);
    Ok(res)
}
