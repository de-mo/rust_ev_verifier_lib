use super::super::{
    result::{create_verification_failure, VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
};
use crate::{
    config::Config,
    file_structure::{
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::meta_data::VerificationMetaDataList,
};
use anyhow::anyhow;
use log::debug;

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> VerificationList<'a> {
    VerificationList(vec![Verification::new(
        "09.01",
        fn_verification_0901,
        metadata_list,
        config,
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

fn fn_verification_0901<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_tally();
    for d in setup_dir.bb_directories().iter() {
        validate_bb_dir(d, result);
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::result::{VerificationResult, VerificationResultTrait},
        *,
    };
    use crate::config::test::{get_test_verifier_tally_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0901(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
