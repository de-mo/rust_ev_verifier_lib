mod v0501_0502_encryption_parameters_payload;
mod v0503_voting_options;
mod v0504_key_generation_schnorr_proofs;
mod v0521_encrypted_pcc_exponentiation_proofs;

use super::super::{suite::VerificationList, verifications::Verification};
use crate::{config::Config, verification::meta_data::VerificationMetaDataList};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> VerificationList<'a> {
    VerificationList(vec![
        Verification::new(
            "05.01",
            "VerifyEncryptionParameters",
            v0501_0502_encryption_parameters_payload::fn_0501_verify_encryption_parameters,
            metadata_list,
            config,
        )
        .unwrap(),
        Verification::new(
            "05.02",
            "VerifySmallPrimeGroupMembers",
            v0501_0502_encryption_parameters_payload::fn_0502_verify_small_prime_group_members,
            metadata_list,
            config,
        )
        .unwrap(),
        Verification::new(
            "05.03",
            "VerifyVotingOptions",
            v0503_voting_options::fn_verification,
            metadata_list,
            config,
        )
        .unwrap(),
        Verification::new(
            "05.04",
            "VerifySchnorrProofs",
            v0504_key_generation_schnorr_proofs::fn_verification,
            metadata_list,
            config,
        )
        .unwrap(),
        Verification::new(
            "05.21",
            "VerifyEncryptedPccExponentiationProofs",
            v0521_encrypted_pcc_exponentiation_proofs::fn_verification,
            metadata_list,
            config,
        )
        .unwrap(),
    ])
}
