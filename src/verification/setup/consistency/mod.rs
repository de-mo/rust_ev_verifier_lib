pub mod v300_encryption_group_consistency;
pub mod v301_setup_file_names_consistency;
pub mod v302_ccr_choice_return_codes_pk_consistency;
pub mod v303_ccm_election_pk_consistency;
pub mod v304_ccm_and_ccr_schnorr_proofs_consistency;
pub mod v305_choice_return_codes_public_key_consistency;
pub mod v306_election_pk_consistency;

use super::super::VerificationList;

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(v300_encryption_group_consistency::get_verification());
    res.push(v301_setup_file_names_consistency::get_verification());
    res.push(v302_ccr_choice_return_codes_pk_consistency::get_verification());
    res.push(v303_ccm_election_pk_consistency::get_verification());
    res.push(v304_ccm_and_ccr_schnorr_proofs_consistency::get_verification());
    res.push(v305_choice_return_codes_public_key_consistency::get_verification());
    res.push(v306_election_pk_consistency::get_verification());
    res
}
