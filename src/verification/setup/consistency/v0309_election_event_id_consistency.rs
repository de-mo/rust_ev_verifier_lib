use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use anyhow::anyhow;
use log::debug;

fn test_election_event_id(
    ee_id: &String,
    expected: &String,
    name: &str,
    result: &mut VerificationResult,
) {
    if ee_id != expected {
        result.push(create_verification_failure!(format!(
            "Election Event ID not equal in {}",
            name
        )));
    }
}

fn test_ee_id_for_context_vcs_dir<V: ContextVCSDirectoryTrait>(
    dir: &V,
    expected: &String,
    result: &mut VerificationResult,
) {
    match dir.setup_component_tally_data_payload() {
        Ok(p) => test_election_event_id(
            &p.election_event_id,
            expected,
            &format!("{}/setup_component_tally_data_payload", dir.get_name()),
            result,
        ),
        Err(e) => result.push(create_verification_error!(
            format!(
                "{}/setup_component_tally_data_payload has wrong format",
                dir.get_name()
            ),
            e
        )),
    }
}

fn test_ee_id_for_setup_vcs_dir<V: SetupVCSDirectoryTrait>(
    dir: &V,
    expected: &String,
    result: &mut VerificationResult,
) {
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        match f {
            Err(e) => result.push(create_verification_error!(
                format!(
                    "{}/control_component_code_shares_payload_.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                e
            )),
            Ok(cc) => {
                for p in cc.0.iter() {
                    test_election_event_id(
                        &p.election_event_id,
                        expected,
                        &format!(
                            "{}/control_component_code_shares_payload.{}_chunk{}",
                            dir.get_name(),
                            i,
                            p.chunk_id
                        ),
                        result,
                    )
                }
            }
        }
    }
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        match f {
            Err(e) => result.push(create_verification_error!(
                format!(
                    "{}/setup_component_verification_data_payload.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                e
            )),
            Ok(s) => test_election_event_id(
                &s.election_event_id,
                expected,
                &format!(
                    "{}/setup_component_verification_data_payload.{}",
                    i,
                    dir.get_name()
                ),
                result,
            ),
        }
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_dir = dir.unwrap_setup();
    let ee_id = match context_dir.election_event_context_payload() {
        Ok(o) => o.election_event_context.election_event_id,
        Err(e) => {
            result.push(create_verification_error!(
                "Cannot extract election_event_context_payload",
                e
            ));
            return;
        }
    };
    match context_dir.setup_component_public_keys_payload() {
        Ok(p) => test_election_event_id(
            &p.election_event_id,
            &ee_id,
            "setup_component_public_keys_payload",
            result,
        ),
        Err(e) => result.push(create_verification_error!(
            "election_event_context_payload has wrong format",
            e
        )),
    }
    for (i, f) in context_dir.control_component_public_keys_payload_iter() {
        match f {
            Err(e) => result.push(create_verification_error!(
                format!(
                    "control_component_public_keys_payload.{} has wrong format",
                    i
                ),
                e
            )),
            Ok(cc) => test_election_event_id(
                &cc.election_event_id,
                &ee_id,
                &format!("control_component_public_keys_payload.{}", i),
                result,
            ),
        }
    }
    for vcs in context_dir.vcs_directories().iter() {
        test_ee_id_for_context_vcs_dir(vcs, &ee_id, result);
    }
    for vcs in setup_dir.vcs_directories().iter() {
        test_ee_id_for_setup_vcs_dir(vcs, &ee_id, result);
    }
}

#[cfg(test)]
mod test {
    use super::{super::super::super::result::VerificationResultTrait, *};
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
