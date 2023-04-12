pub mod v500_501_encryption_parameters_payload;
pub mod v502_voting_options;
pub mod v503_key_generation_schnorr_proofs;

use crate::verification::meta_data::VerificationMetaDataList;

use super::super::{verification::Verification, VerificationSuite};

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationSuite {
    let mut res = vec![];
    res.push(
        Verification::new(
            "s500",
            v500_501_encryption_parameters_payload::fn_verification_500,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s501",
            v500_501_encryption_parameters_payload::fn_verification_501,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new("s502", v502_voting_options::fn_verification, metadata_list).unwrap(),
    );
    res.push(
        Verification::new(
            "s503",
            v503_key_generation_schnorr_proofs::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res
}
