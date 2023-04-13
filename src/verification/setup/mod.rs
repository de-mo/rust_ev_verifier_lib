//! Module implementing the verifications for setup
pub mod authenticity;
pub mod completness;
pub mod consistency;
pub mod evidence;
pub mod integrity;

use super::{meta_data::VerificationMetaDataList, verification_suite::VerificationList};

/// Collect the verifications of the submodules
pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res: VerificationList = vec![];
    res.append(&mut authenticity::get_verifications(metadata_list));
    res.append(&mut completness::get_verifications(metadata_list));
    res.append(&mut consistency::get_verifications(metadata_list));
    res.append(&mut evidence::get_verifications(metadata_list));
    res.append(&mut integrity::get_verifications(metadata_list));
    res
}
