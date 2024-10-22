mod v0810_verify_file_name_node_ids_consistency;
mod v0811_verify_encryption_group_consistency;

use super::super::{suite::VerificationList, verifications::Verification};
use crate::{
    config::Config,
    verification::{
        meta_data::VerificationMetaDataList, verification_unimplemented, VerificationError,
    },
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![
        Verification::new(
            "08.01",
            "VerifyConfirmedEncryptedVotesConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.02",
            "VerifyCiphertextsConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.03",
            "VerifyPlaintextsConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.04",
            "VerifyVerificationCardIdsConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.05",
            "VerifyBallotBoxIdsConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.06",
            "VerifyFileNameBallotBoxIdsConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.07",
            "VerifyNumberConfirmedEncryptedVotesConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.08",
            "VerifyElectionEventIdConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.09",
            "VerifyNodeIdsConsistency",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.10",
            "VerifyFileNameNodeIdsConsistency",
            v0810_verify_file_name_node_ids_consistency::fn_verification,
            metadata_list,
            config,
        )?,
        Verification::new(
            "08.11",
            "VerifyEncryptionGroupConsistency",
            v0811_verify_encryption_group_consistency::fn_verification,
            metadata_list,
            config,
        )?,
    ]))
}
