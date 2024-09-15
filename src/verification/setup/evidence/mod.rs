mod v0501_encryption_parameters;
mod v0502_verify_small_prime_group_members;
mod v0503_voting_options;
mod v0504_key_generation_schnorr_proofs;
mod v0521_verify_signature_verification_data_and_code_proofs;

use super::super::{suite::VerificationList, verifications::Verification};
use crate::{
    config::Config,
    verification::{meta_data::VerificationMetaDataList, VerificationError},
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![
        Verification::new(
            "05.01",
            "VerifyEncryptionParameters",
            v0501_encryption_parameters::fn_0501_verify_encryption_parameters,
            metadata_list,
            config,
        )?,
        Verification::new(
            "05.02",
            "VerifySmallPrimeGroupMembers",
            v0502_verify_small_prime_group_members::fn_0502_verify_small_prime_group_members,
            metadata_list,
            config,
        )?,
        Verification::new(
            "05.03",
            "VerifyVotingOptions",
            v0503_voting_options::fn_verification,
            metadata_list,
            config,
        )?,
        Verification::new(
            "05.04",
            "VerifySchnorrProofs",
            v0504_key_generation_schnorr_proofs::fn_verification,
            metadata_list,
            config,
        )?,
        Verification::new(
            "05.21",
            "VerifySignatureVerificationDataAndCodeProofs",
            v0521_verify_signature_verification_data_and_code_proofs::fn_verification,
            metadata_list,
            config,
        )?,
    ]))
}
