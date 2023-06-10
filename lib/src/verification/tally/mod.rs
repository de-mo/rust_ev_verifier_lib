pub mod authenticity;
pub mod completness;
pub mod consistency;
pub mod evidence;
pub mod integrity;

use super::{meta_data::VerificationMetaDataList, suite::VerificationList};

pub fn get_verifications<'a>(metadata_list: &'a VerificationMetaDataList) -> VerificationList<'a> {
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
