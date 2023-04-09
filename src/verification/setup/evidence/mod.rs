pub mod v500_501_encryption_parameters_payload;
pub mod v502_voting_options;
pub mod v503_key_generation_schnorr_proofs;

use super::super::VerificationList;

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(v500_501_encryption_parameters_payload::get_verification_500());
    res.push(v500_501_encryption_parameters_payload::get_verification_501());
    res.push(v502_voting_options::get_verification());
    res.push(v503_key_generation_schnorr_proofs::get_verification());
    res
}
