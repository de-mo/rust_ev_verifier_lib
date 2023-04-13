use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::{setup_directory::VCSDirectory, VerificationDirectory},
    verification::meta_data::VerificationMetaDataList,
};

use super::super::{
    error::{create_verification_failure, VerificationFailureType},
    verification::{Verification, VerificationResult},
    verification_suite::VerificationList,
};

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res = vec![];
    res.push(Verification::new("s400", fn_verification_400, metadata_list).unwrap());
    res
}

fn validate_vcs_dir(dir: &VCSDirectory, result: &mut VerificationResult) {
    match dir.setup_component_tally_data_payload() {
        Ok(_) => (),
        Err(e) => result.push_failure(create_verification_failure!(
            format!(
                "{}/setup_component_tally_data_payload has wrong format",
                dir.get_name()
            ),
            e
        )),
    }
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        if f.is_err() {
            result.push_failure(create_verification_failure!(
                format!(
                    "{}/control_component_code_shares_payload.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                f.unwrap_err()
            ))
        }
    }
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        if f.is_err() {
            result.push_failure(create_verification_failure!(
                format!(
                    "{}/setup_component_verification_data_payload.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                f.unwrap_err()
            ))
        }
    }
}

fn fn_verification_400(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    match setup_dir.encryption_parameters_payload() {
        Ok(_) => (),
        Err(e) => result.push_failure(create_verification_failure!(
            "encryption_parameters_payload has wrong format",
            e
        )),
    }
    match setup_dir.election_event_context_payload() {
        Ok(_) => (),
        Err(e) => result.push_failure(create_verification_failure!(
            "election_event_context_payload has wrong format",
            e
        )),
    }
    match setup_dir.setup_component_public_keys_payload() {
        Ok(_) => (),
        Err(e) => result.push_failure(create_verification_failure!(
            "setup_component_public_keys_payload has wrong format",
            e
        )),
    }
    match setup_dir.election_event_configuration() {
        Ok(_) => (),
        Err(e) => result.push_failure(create_verification_failure!(
            "election_event_configuration has wrong format",
            e
        )),
    }
    for (i, f) in setup_dir.control_component_public_keys_payload_iter() {
        if f.is_err() {
            result.push_failure(create_verification_failure!(
                format!(
                    "control_component_public_keys_payload.{} has wrong format",
                    i
                ),
                f.unwrap_err()
            ))
        }
    }
    for d in setup_dir.vcs_directories_iter() {
        validate_vcs_dir(d, result);
    }
}

#[cfg(test)]
mod test {

    use super::super::super::verification::VerificationResultTrait;
    use super::*;
    use crate::verification::VerificationPeriod;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_400(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
