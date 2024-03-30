mod authenticity;
mod completness;
mod consistency;
mod evidence;
mod integrity;

use crate::config::Config;

use super::{meta_data::VerificationMetaDataList, suite::VerificationList};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> anyhow::Result<VerificationList<'a>> {
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
