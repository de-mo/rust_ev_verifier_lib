pub mod verify_ccm_and_ccr_schnorr_proofs_consistency;
pub mod verify_ccm_election_pk_consistency;
pub mod verify_ccr_choice_return_codes_pk_consistency;
pub mod verify_choice_return_codes_public_key_consistency;
pub mod verify_encryption_group_consistency;
pub mod verify_setup_file_names_consistency;

use super::super::VerificationList;

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(verify_encryption_group_consistency::get_verification_300());
    res.push(verify_setup_file_names_consistency::get_verification_301());
    res.push(verify_ccr_choice_return_codes_pk_consistency::get_verification_302());
    res.push(verify_ccm_election_pk_consistency::get_verification_303());
    res.push(verify_ccm_and_ccr_schnorr_proofs_consistency::get_verification_304());
    res.push(verify_choice_return_codes_public_key_consistency::get_verification_305());
    res
}
