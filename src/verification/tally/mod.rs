pub mod authenticity;
pub mod completness;
pub mod consistency;
pub mod evidence;
pub mod integrity;

use std::collections::HashMap;

use super::{VerificationCategory, VerificationListCategory};

pub fn get_verifications() -> VerificationListCategory {
    let mut res: VerificationListCategory = HashMap::new();
    res.insert(
        VerificationCategory::Authenticity,
        authenticity::get_verifications(),
    );
    res.insert(
        VerificationCategory::Completness,
        completness::get_verifications(),
    );
    res.insert(
        VerificationCategory::Consistency,
        consistency::get_verifications(),
    );
    res.insert(
        VerificationCategory::Evidence,
        evidence::get_verifications(),
    );
    res.insert(
        VerificationCategory::Integrity,
        integrity::get_verifications(),
    );
    res
}
