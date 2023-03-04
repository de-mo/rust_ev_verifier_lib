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
        Box::new(authenticity::get_verifications()),
    );
    res.insert(
        VerificationCategory::Completness,
        Box::new(completness::get_verifications()),
    );
    res.insert(
        VerificationCategory::Consistency,
        Box::new(consistency::get_verifications()),
    );
    res.insert(
        VerificationCategory::Evidence,
        Box::new(evidence::get_verifications()),
    );
    res.insert(
        VerificationCategory::Integrity,
        Box::new(integrity::get_verifications()),
    );
    res
}
