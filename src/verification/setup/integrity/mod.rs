use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::{setup_directory::VCSDirectory, VerificationDirectory},
};

use super::super::{
    error::{
        create_verification_failure, VerificationError, VerificationFailure,
        VerificationFailureType,
    },
    verification::{Verification, VerificationMetaData},
    VerificationCategory, VerificationList, VerificationPeriod,
};

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(get_verification_400());
    res
}

fn get_verification_400() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "400".to_owned(),
            nr: "3.4".to_owned(),
            name: "Integrity".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Integrity,
        },
        fn_verification_400,
    )
}

fn validate_vcs_dir(dir: &VCSDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut failures: Vec<VerificationFailure> = vec![];
    match dir.setup_component_tally_data_payload() {
        Ok(_) => (),
        Err(e) => failures.push(create_verification_failure!(
            format!(
                "{}/setup_component_tally_data_payload has wrong format",
                dir.get_name()
            ),
            e
        )),
    }
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        if f.is_err() {
            failures.push(create_verification_failure!(
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
            failures.push(create_verification_failure!(
                format!(
                    "{}/setup_component_verification_data_payload.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                f.unwrap_err()
            ))
        }
    }
    (vec![], failures)
}

fn fn_verification_400(
    dir: &VerificationDirectory,
) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut errors: Vec<VerificationError> = vec![];
    let mut failures: Vec<VerificationFailure> = vec![];
    let setup_dir = dir.unwrap_setup();
    match setup_dir.encryption_parameters_payload() {
        Ok(_) => (),
        Err(e) => failures.push(create_verification_failure!(
            "encryption_parameters_payload has wrong format",
            e
        )),
    }
    match setup_dir.election_event_context_payload() {
        Ok(_) => (),
        Err(e) => failures.push(create_verification_failure!(
            "election_event_context_payload has wrong format",
            e
        )),
    }
    match setup_dir.setup_component_public_keys_payload() {
        Ok(_) => (),
        Err(e) => failures.push(create_verification_failure!(
            "setup_component_public_keys_payload has wrong format",
            e
        )),
    }
    for (i, f) in setup_dir.control_component_public_keys_payload_iter() {
        if f.is_err() {
            failures.push(create_verification_failure!(
                format!(
                    "control_component_public_keys_payload.{} has wrong format",
                    i
                ),
                f.unwrap_err()
            ))
        }
    }
    for d in setup_dir.vcs_directories_iter() {
        let (mut es, mut fs) = validate_vcs_dir(d);
        errors.append(&mut es);
        failures.append(&mut fs);
    }
    (errors, failures)
}

#[cfg(test)]
mod test {
    use crate::file_structure::setup_directory::SetupDirectory;

    use super::*;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::Setup(SetupDirectory::new(&location))
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let (e, f) = fn_verification_400(&dir);
        assert!(e.is_empty());
        assert!(f.is_empty());
    }
}
