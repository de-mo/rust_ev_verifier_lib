use crate::{
    file_structure::{
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::meta_data::VerificationMetaDataList,
};

use super::super::{
    result::{create_verification_failure, VerificationEvent, VerificationResult},
    suite::VerificationList,
    verification::Verification,
};
use anyhow::anyhow;
use log::debug;

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res = vec![];
    res.push(Verification::new("06.01", fn_verification_0601, metadata_list).unwrap());
    res
}

fn validate_bb_dir<B: BBDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    if !dir.tally_component_shuffle_payload_file().exists() {
        result.push(create_verification_failure!(
            "tally_component_shuffle_payload does not exist"
        ))
    }
    if !dir.tally_component_shuffle_payload_file().exists() {
        result.push(create_verification_failure!(
            "tally_component_shuffle_payload does not exist"
        ))
    }
    if !dir
        .control_component_ballot_box_payload_group()
        .has_elements()
    {
        result.push(create_verification_failure!(
            "control_component_ballot_box_payload does not exist"
        ))
    }
    if !dir.control_component_shuffle_payload_group().has_elements() {
        result.push(create_verification_failure!(
            "control_component_shuffle_payload does not exist"
        ))
    }
}

fn fn_verification_0601<D: VerificationDirectoryTrait>(dir: &D, result: &mut VerificationResult) {
    let tally_dir = dir.unwrap_tally();
    if !tally_dir.ech_0110_file().exists() {
        result.push(create_verification_failure!("ech_0110 does not exist"))
    }
    if !tally_dir.ech_0222_file().exists() {
        result.push(create_verification_failure!("ech_0222 does not exist"))
    }
    if !tally_dir.e_voting_decrypt_file().exists() {
        result.push(create_verification_failure!(
            "e_voting_decrypt does not exist"
        ))
    }
    for d in tally_dir.bb_directories().iter() {
        validate_bb_dir(d, result);
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::{result::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::file_structure::VerificationDirectory;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset1-setup-tally");
        VerificationDirectory::new(&VerificationPeriod::Tally, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0601(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
