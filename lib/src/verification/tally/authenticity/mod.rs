use crate::{config::Config, verification::meta_data::VerificationMetaDataList};

use super::super::suite::VerificationList;

pub fn get_verifications<'a>(
    _metadata_list: &'a VerificationMetaDataList,
    _config: &'static Config,
) -> VerificationList<'a> {
    let res = vec![];
    VerificationList(res)
}
