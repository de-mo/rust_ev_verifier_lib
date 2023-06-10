use super::super::{
    result::{create_verification_failure, VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
};
use crate::{
    file_structure::{
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::meta_data::VerificationMetaDataList,
};
use anyhow::anyhow;
use log::debug;

pub(crate) fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    VerificationList(vec![Verification::new(
        "09.01",
        fn_verification_0901,
        metadata_list,
    )
    .unwrap()])
}

fn validate_bb_dir<B: BBDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    match dir.tally_component_votes_payload() {
        Ok(_) => (),
        Err(e) => result.push(create_verification_failure!(
            format!(
                "{}/tally_component_votes_payload has wrong format",
                dir.get_name()
            ),
            e
        )),
    }
    match dir.tally_component_shuffle_payload() {
        Ok(_) => (),
        Err(e) => result.push(create_verification_failure!(
            format!(
                "{}/tally_component_shuffle_payload has wrong format",
                dir.get_name()
            ),
            e
        )),
    }
    for (i, f) in dir.control_component_ballot_box_payload_iter() {
        if let Err(e) = f {
            result.push(create_verification_failure!(
                format!(
                    "{}/control_component_ballot_box_payload_iter.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                e
            ))
        }
    }
    for (i, f) in dir.control_component_shuffle_payload_iter() {
        if let Err(e) = f {
            result.push(create_verification_failure!(
                format!(
                    "{}/control_component_shuffle_payload_iter.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                e
            ))
        }
    }
}

fn fn_verification_0901<D: VerificationDirectoryTrait>(dir: &D, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_tally();
    for d in setup_dir.bb_directories().iter() {
        validate_bb_dir(d, result);
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::{
            result::{VerificationResult, VerificationResultTrait},
            VerificationPeriod,
        },
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
        fn_verification_0901(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
