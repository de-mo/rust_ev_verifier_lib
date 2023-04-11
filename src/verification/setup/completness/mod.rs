use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::{setup_directory::VCSDirectory, VerificationDirectory},
};

use super::super::{
    error::{create_verification_failure, VerificationFailureType},
    verification::{Verification, VerificationMetaData, VerificationResult},
    VerificationCategory, VerificationList, VerificationPeriod,
};

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(get_verification_100());
    res
}

fn get_verification_100() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "100".to_owned(),
            algorithm: "3.1".to_owned(),
            name: "VerifySetupCompleteness".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Completness,
        },
        fn_verification_100,
    )
}

fn validate_vcs_dir(dir: &VCSDirectory, result: &mut VerificationResult) {
    if !dir.setup_component_tally_data_payload_file.exists() {
        result.push_failure(create_verification_failure!(
            "setup_component_tally_data_payload does not exist"
        ))
    }
    if !dir
        .setup_component_verification_data_payload_group
        .has_elements()
    {
        result.push_failure(create_verification_failure!(
            "setup_component_verification_data_payload does not exist"
        ))
    }
    if !dir
        .control_component_code_shares_payload_group
        .has_elements()
    {
        result.push_failure(create_verification_failure!(
            "control_component_code_shares_payload does not exist"
        ))
    }
}

fn fn_verification_100(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    if !setup_dir.encryption_parameters_payload_file.exists() {
        result.push_failure(create_verification_failure!(
            "encryption_parameters_payload does not exist"
        ))
    }
    if !setup_dir.election_event_context_payload_file.exists() {
        result.push_failure(create_verification_failure!(
            "election_event_context_payload does not exist"
        ))
    }
    if !setup_dir.setup_component_public_keys_payload_file.exists() {
        result.push_failure(create_verification_failure!(
            "setup_component_public_keys_payload_file does not exist"
        ))
    }
    if !setup_dir.election_event_configuration_file.exists() {
        result.push_failure(create_verification_failure!(
            "setup_component_public_keys_payload_file does not exist"
        ))
    }
    if setup_dir
        .control_component_public_keys_payload_group
        .get_numbers()
        != vec![1, 2, 3, 4]
    {
        result.push_failure(create_verification_failure!(format!(
            "control_component_public_keys_payload_group missing. only these parts are present: {:?}",
            setup_dir
                .control_component_public_keys_payload_group
                .get_numbers()
        )))
    }
    for d in setup_dir.vcs_directories_iter() {
        validate_vcs_dir(d, result);
    }
}

#[cfg(test)]
mod test {
    use super::super::super::verification::VerificationResultTrait;
    use super::*;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::new(VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_100(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
