//! Module implementing the verifications for setup
mod authenticity;
mod completness;
mod consistency;
mod evidence;
mod integrity;

use super::{meta_data::VerificationMetaDataList, suite::VerificationList};

/// Collect the verifications of the submodules
pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList<'_> {
    let mut res = VerificationList(vec![]);
    res.0
        .append(&mut authenticity::get_verifications(metadata_list).0);
    res.0
        .append(&mut completness::get_verifications(metadata_list).0);
    res.0
        .append(&mut consistency::get_verifications(metadata_list).0);
    res.0
        .append(&mut evidence::get_verifications(metadata_list).0);
    res.0
        .append(&mut integrity::get_verifications(metadata_list).0);
    res
}
