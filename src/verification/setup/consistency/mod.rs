pub mod verify_encryption_group_consistency;
pub mod verify_setup_file_names_consistency;

use super::super::VerificationList;

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(verify_encryption_group_consistency::get_verification_300());
    res.push(verify_setup_file_names_consistency::get_verification_301());
    res
}
