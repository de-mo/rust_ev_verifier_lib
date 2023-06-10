pub mod v0501_0502_encryption_parameters_payload;
pub mod v0503_voting_options;
pub mod v0504_key_generation_schnorr_proofs;
pub mod v0521_encrypted_pcc_exponentiation_proofs;

use super::super::{suite::VerificationList, verification::Verification};
use crate::verification::meta_data::VerificationMetaDataList;

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res = vec![];
    res.push(
        Verification::new(
            "05.01",
            v0501_0502_encryption_parameters_payload::fn_verification_0501,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "05.02",
            v0501_0502_encryption_parameters_payload::fn_verification_0502,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "05.03",
            v0503_voting_options::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "05.04",
            v0504_key_generation_schnorr_proofs::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "05.21",
            v0521_encrypted_pcc_exponentiation_proofs::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    VerificationList(res)
}
