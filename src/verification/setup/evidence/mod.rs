mod v500_501_encryption_parameters_payload;

use super::super::VerificationList;

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(v500_501_encryption_parameters_payload::get_verification_500());
    res.push(v500_501_encryption_parameters_payload::get_verification_501());
    res
}
