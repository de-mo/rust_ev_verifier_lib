use super::super::{
    result::VerificationEvent, suite::VerificationList, verifications::Verification,
};
use crate::{
    config::Config,
    file_structure::{
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::{
        meta_data::VerificationMetaDataList, result::VerificationResult,
        verification_unimplemented, verify_signature_for_object,
    },
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> anyhow::Result<VerificationList<'a>> {
    Ok(VerificationList(vec![
        Verification::new(
            "07.01",
            "VerifySignatureControlComponentBallotBox",
            fn_0701_verify_signature_control_component_ballot_box,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.02",
            "VerifySignatureControlComponentShuffle",
            fn_0702_verify_verify_signature_control_component_shuffle,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.03",
            "VerifySignatureTallyComponentShuffle",
            fn_0703_verify_signature_tally_component_shuffle,
            metadata_list,
            config,
        )?,
        Verification::new(
            "07.04",
            "VerifySignatureTallyComponentVotes",
            fn_0704_verify_signature_tally_component_votes,
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

fn fn_0701_verify_signature_control_component_ballot_box<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    for bb_d in tally_dir.bb_directories().iter() {
        for (i, f) in bb_d.control_component_ballot_box_payload_iter() {
            match f {
                Ok(d) => result.append_with_context(
                    &verify_signature_for_object(d.as_ref(), config),
                    format!(
                        "{}/control_component_ballot_box_payload_{}.json",
                        bb_d.get_name(),
                        i
                    ),
                ),
                Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                    "{}/control_component_ballot_box_payload_{}.json",
                    bb_d.get_name(),
                    i
                ))),
            }
        }
    }
}

fn fn_0702_verify_verify_signature_control_component_shuffle<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    for bb_d in tally_dir.bb_directories().iter() {
        for (i, f) in bb_d.control_component_shuffle_payload_iter() {
            match f {
                Ok(d) => result.append_with_context(
                    &verify_signature_for_object(d.as_ref(), config),
                    format!(
                        "{}/control_component_shuffle_payload_{}.json",
                        bb_d.get_name(),
                        i
                    ),
                ),
                Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                    "{}/control_component_shuffle_payload_{}.json",
                    bb_d.get_name(),
                    i
                ))),
            }
        }
    }
}

fn fn_0703_verify_signature_tally_component_shuffle<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    for bb_d in tally_dir.bb_directories().iter() {
        match bb_d.tally_component_shuffle_payload() {
            Ok(d) => result.append_with_context(
                &verify_signature_for_object(d.as_ref(), config),
                format!("{}/tally_component_shuffle_payload.json", bb_d.get_name(),),
            ),
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/tally_component_shuffle_payload.json",
                bb_d.get_name(),
            ))),
        }
    }
}

fn fn_0704_verify_signature_tally_component_votes<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    for bb_d in tally_dir.bb_directories().iter() {
        match bb_d.tally_component_votes_payload() {
            Ok(d) => result.append_with_context(
                &verify_signature_for_object(d.as_ref(), config),
                format!("{}/tally_component_votes_payload.json", bb_d.get_name(),),
            ),
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/tally_component_votes_payload.json",
                bb_d.get_name(),
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_tally_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_0701() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0701_verify_signature_control_component_ballot_box(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_0702() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0702_verify_verify_signature_control_component_shuffle(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Signature for tally_component_shuffle_payload not working"]
    fn test_0703() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0703_verify_signature_tally_component_shuffle(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_0704() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0704_verify_signature_tally_component_votes(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }
}
