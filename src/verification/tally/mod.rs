pub mod authenticity;
pub mod completness;
pub mod consistency;
pub mod evidence;
pub mod integrity;

use super::{meta_data::VerificationMetaDataList, VerificationSuite};

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationSuite {
    let mut res: VerificationSuite = vec![];
    res.append(&mut authenticity::get_verifications(metadata_list));
    res.append(&mut completness::get_verifications(metadata_list));
    res.append(&mut consistency::get_verifications(metadata_list));
    res.append(&mut evidence::get_verifications(metadata_list));
    res.append(&mut integrity::get_verifications(metadata_list));
    res
}
