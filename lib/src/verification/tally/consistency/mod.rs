use crate::verification::meta_data::VerificationMetaDataList;

use super::super::suite::VerificationList;

pub(crate) fn get_verifications(_: &VerificationMetaDataList) -> VerificationList {
    let res = vec![];
    VerificationList(res)
}
