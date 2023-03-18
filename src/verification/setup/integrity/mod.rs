use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::{setup_directory::VCSDirectory, VerificationDirectory},
    verification::error::{
        create_verification_failure, VerificationError, VerificationFailure,
        VerificationFailureType,
    },
};

use super::super::{
    verification::{Verification, VerificationMetaData},
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
            id: "400".to_owned(),
            nr: "3.4".to_owned(),
            name: "Integrity".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Completness,
        },
        fn_verification_400,
    )
}

fn validate_vcs_dir(dir: &VCSDirectory) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut failures: Vec<VerificationFailure> = vec![];
    if !dir.setup_component_tally_data_payload_file.exists() {
        failures.push(create_verification_failure!(
            "setup_component_tally_data_payload does not exist"
        ))
    }
    if !dir
        .setup_component_verification_data_payload_group
        .has_elements()
    {
        failures.push(create_verification_failure!(
            "setup_component_verification_data_payload does not exist"
        ))
    }
    if !dir
        .control_component_code_shares_payload_group
        .has_elements()
    {
        failures.push(create_verification_failure!(
            "control_component_code_shares_payload does not exist"
        ))
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
    for f in setup_dir.control_component_public_keys_payload_iter() {}
    {
        failures.push(create_verification_failure!(format!(
            "control_component_public_keys_payload_group missing. only these parts are present: {:?}",
            setup_dir
                .control_component_public_keys_payload_group
                .get_numbers()
        )))
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
