pub mod VerifyEncryptionGroupConsistency;

use super::super::VerificationList;

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(VerifyEncryptionGroupConsistency::get_verification_300());
    res
}
