use super::super::{suite::VerificationList, verifications::Verification};
use crate::{
    config::Config,
    verification::{meta_data::VerificationMetaDataList, verification_unimplemented},
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> anyhow::Result<VerificationList<'a>> {
    Ok(VerificationList(vec![
        Verification::new(
            "07.01",
            "VerifySignatureControlComponentBallotBox",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.02",
            "VerifySignatureControlComponentShuffle",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.03",
            "VerifySignatureTallyComponentShuffle",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.04",
            "VerifySignatureTallyComponentVotes",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.05",
            "VerifySignatureTallyComponentDecrypt",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.06",
            "VerifySignatureTallyComponentEch0222",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.07",
            "VerifySignatureTallyComponentEch0110",
            verification_unimplemented,
            metadata_list,
            config,
        )?,
    ]))
}
