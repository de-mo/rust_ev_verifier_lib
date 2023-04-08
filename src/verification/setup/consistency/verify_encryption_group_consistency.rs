use crate::{
    data_structures::setup::encryption_parameters_payload::EncryptionGroup,
    error::{create_verifier_error, VerifierError},
    file_structure::setup_directory::VCSDirectory,
};

use crate::file_structure::VerificationDirectory;

use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationError,
        VerificationErrorType, VerificationFailure, VerificationFailureType,
    },
    verification::{Verification, VerificationMetaData},
    VerificationCategory, VerificationPeriod,
};

pub(super) fn get_verification_300() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "300".to_owned(),
            nr: "3.01".to_owned(),
            name: "VerifyEncryptionGroupConsistency".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Consistency,
        },
        fn_verification_300,
    )
}

fn test_encryption_group(
    eg: &EncryptionGroup,
    expected: &EncryptionGroup,
    name: &str,
) -> Vec<VerificationFailure> {
    let mut res = vec![];
    if eg.p != expected.p {
        res.push(create_verification_failure!(format!(
            "p not equal in {}",
            name
        )));
    }
    if eg.q != expected.q {
        res.push(create_verification_failure!(format!(
            "q not equal in {}",
            name
        )));
    }
    if eg.g != expected.g {
        res.push(create_verification_failure!(format!(
            "g not equal in {}",
            name
        )));
    }
    res
}

fn test_encryption_group_for_vcs_dir(
    dir: &VCSDirectory,
    eg: &EncryptionGroup,
) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut errors: Vec<VerificationError> = vec![];
    let mut failures: Vec<VerificationFailure> = vec![];
    match dir.setup_component_tally_data_payload() {
        Ok(p) => failures.append(&mut test_encryption_group(
            &p.encryption_group,
            &eg,
            &format!("{}/setup_component_tally_data_payload", dir.get_name()),
        )),
        Err(e) => errors.push(create_verification_error!(
            format!(
                "{}/setup_component_tally_data_payload has wrong format",
                dir.get_name()
            ),
            e
        )),
    }
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        if f.is_err() {
            errors.push(create_verification_error!(
                format!(
                    "{}/control_component_code_shares_payload_.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                f.unwrap_err()
            ))
        } else {
            for p in f.unwrap().iter() {
                failures.append(&mut test_encryption_group(
                    &p.encryption_group,
                    &eg,
                    &format!(
                        "{}/control_component_code_shares_payload.{}_chunk{}",
                        dir.get_name(),
                        i,
                        p.chunk_id
                    ),
                ))
            }
        }
    }
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        if f.is_err() {
            errors.push(create_verification_error!(
                format!(
                    "{}/setup_component_verification_data_payload.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                f.unwrap_err()
            ))
        } else {
            failures.append(&mut test_encryption_group(
                &f.unwrap().encryption_group,
                &eg,
                &format!(
                    "{}/setup_component_verification_data_payload.{}",
                    i,
                    dir.get_name()
                ),
            ))
        }
    }
    (errors, failures)
}

fn fn_verification_300(
    dir: &VerificationDirectory,
) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut errors: Vec<VerificationError> = vec![];
    let mut failures: Vec<VerificationFailure> = vec![];
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(p) => p.encryption_group,
        Err(e) => {
            return (
                vec![create_verification_error!(
                    "encryption_parameters_payload cannot be read",
                    e
                )],
                failures,
            )
        }
    };
    match setup_dir.election_event_context_payload() {
        Ok(p) => failures.append(&mut test_encryption_group(
            &p.encryption_group,
            &eg,
            "election_event_context_payload",
        )),
        Err(e) => errors.push(create_verification_error!(
            "election_event_context_payload has wrong format",
            e
        )),
    }
    match setup_dir.setup_component_public_keys_payload() {
        Ok(p) => failures.append(&mut test_encryption_group(
            &p.encryption_group,
            &eg,
            "setup_component_public_keys_payload",
        )),
        Err(e) => errors.push(create_verification_error!(
            "election_event_context_payload has wrong format",
            e
        )),
    }
    for (i, f) in setup_dir.control_component_public_keys_payload_iter() {
        if f.is_err() {
            errors.push(create_verification_error!(
                format!(
                    "control_component_public_keys_payload.{} has wrong format",
                    i
                ),
                f.unwrap_err()
            ))
        } else {
            failures.append(&mut test_encryption_group(
                &f.unwrap().encryption_group,
                &eg,
                &format!("control_component_public_keys_payload.{}", i),
            ))
        }
    }
    for vcs in setup_dir.vcs_directories_iter() {
        let (mut e, mut f) = test_encryption_group_for_vcs_dir(vcs, &eg);
        errors.append(&mut e);
        failures.append(&mut f);
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
        let (e, f) = fn_verification_300(&dir);
        assert!(e.is_empty());
        assert!(f.is_empty());
    }
}
