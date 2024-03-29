use super::super::{
    result::{create_verification_failure, VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
};
use crate::{
    config::Config,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
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
        "01.01",
        "VerifySetupCompleteness",
        fn_0101_verify_setup_completeness,
        metadata_list,
        config,
    )
    .unwrap()])
}

fn validate_context_vcs_dir<B: ContextVCSDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    if !dir.setup_component_tally_data_payload_file().exists() {
        result.push(create_verification_failure!(
            "setup_component_tally_data_payload does not exist"
        ))
    }
}

fn validate_setup_vcs_dir<B: SetupVCSDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    if !dir
        .setup_component_verification_data_payload_group()
        .has_elements()
    {
        result.push(create_verification_failure!(
            "setup_component_verification_data_payload does not exist"
        ))
    }
    if !dir
        .control_component_code_shares_payload_group()
        .has_elements()
    {
        result.push(create_verification_failure!(
            "control_component_code_shares_payload does not exist"
        ))
    }
}

fn validate_context_dir<B: ContextDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    if !dir.election_event_context_payload_file().exists() {
        result.push(create_verification_failure!(
            "election_event_context_payload does not exist"
        ))
    }
    if !dir.setup_component_public_keys_payload_file().exists() {
        result.push(create_verification_failure!(
            "setup_component_public_keys_payload_file does not exist"
        ))
    }
    if !dir.election_event_configuration_file().exists() {
        result.push(create_verification_failure!(
            "setup_component_public_keys_payload_file does not exist"
        ))
    }
    if dir
        .control_component_public_keys_payload_group()
        .get_numbers()
        != &vec![1, 2, 3, 4]
    {
        result.push(create_verification_failure!(format!(
            "control_component_public_keys_payload_group missing. only these parts are present: {:?}",
            dir
                .control_component_public_keys_payload_group()
                .get_numbers()
        )))
    }
    for d in dir.vcs_directories().iter() {
        validate_context_vcs_dir(d, result);
    }
}

fn validate_setup_dir<B: SetupDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    for d in dir.vcs_directories().iter() {
        validate_setup_vcs_dir(d, result);
    }
}

fn fn_0101_verify_setup_completeness<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    validate_context_dir(context_dir, result);
    let setup_dir = dir.unwrap_setup();
    validate_setup_dir(setup_dir, result);
}

#[cfg(test)]
mod test {
    use super::{super::super::result::VerificationResultTrait, *};
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0101_verify_setup_completeness(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
