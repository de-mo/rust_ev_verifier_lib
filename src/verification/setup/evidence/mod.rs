pub mod v500_501_encryption_parameters_payload;
pub mod v502_voting_options;

use super::super::VerificationList;

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(v500_501_encryption_parameters_payload::get_verification_500());
    res.push(v500_501_encryption_parameters_payload::get_verification_501());
    res.push(v502_voting_options::get_verification());
    res
}
