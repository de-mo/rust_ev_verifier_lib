use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    data_structures::common_types::EncryptionGroup,
    error::{create_verifier_error, VerifierError},
    file_structure::{
        setup_directory::{SetupDirectoryTrait, VCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};

fn test_encryption_group(
    eg: &EncryptionGroup,
    expected: &EncryptionGroup,
    name: &str,
    result: &mut VerificationResult,
) {
    if eg.p != expected.p {
        result.push_failure(create_verification_failure!(format!(
            "p not equal in {}",
            name
        )));
    }
    if eg.q != expected.q {
        result.push_failure(create_verification_failure!(format!(
            "q not equal in {}",
            name
        )));
    }
    if eg.g != expected.g {
        result.push_failure(create_verification_failure!(format!(
            "g not equal in {}",
            name
        )));
    }
}

fn test_encryption_group_for_vcs_dir<V: VCSDirectoryTrait>(
    dir: &V,
    eg: &EncryptionGroup,
    result: &mut VerificationResult,
) {
    match dir.setup_component_tally_data_payload() {
        Ok(p) => test_encryption_group(
            &p.encryption_group,
            &eg,
            &format!("{}/setup_component_tally_data_payload", dir.get_name()),
            result,
        ),
        Err(e) => result.push_error(create_verification_error!(
            format!(
                "{}/setup_component_tally_data_payload has wrong format",
                dir.get_name()
            ),
            e
        )),
    }
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        if f.is_err() {
            result.push_error(create_verification_error!(
                format!(
                    "{}/control_component_code_shares_payload_.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                f.unwrap_err()
            ))
        } else {
            for p in f.unwrap().iter() {
                test_encryption_group(
                    &p.encryption_group,
                    &eg,
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
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        if f.is_err() {
            result.push_error(create_verification_error!(
                format!(
                    "{}/setup_component_verification_data_payload.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                f.unwrap_err()
            ))
        } else {
            test_encryption_group(
                &f.unwrap().encryption_group,
                &eg,
                &format!(
                    "{}/setup_component_verification_data_payload.{}",
                    i,
                    dir.get_name()
                ),
                result,
            )
        }
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(p) => p.encryption_group,
        Err(e) => {
            result.push_error(create_verification_error!(
                "encryption_parameters_payload cannot be read",
                e
            ));
            return;
        }
    };
    match setup_dir.election_event_context_payload() {
        Ok(p) => test_encryption_group(
            &p.encryption_group,
            &eg,
            "election_event_context_payload",
            result,
        ),
        Err(e) => result.push_error(create_verification_error!(
            "election_event_context_payload has wrong format",
            e
        )),
    }
    match setup_dir.setup_component_public_keys_payload() {
        Ok(p) => test_encryption_group(
            &p.encryption_group,
            &eg,
            "setup_component_public_keys_payload",
            result,
        ),
        Err(e) => result.push_error(create_verification_error!(
            "election_event_context_payload has wrong format",
            e
        )),
    }
    for (i, f) in setup_dir.control_component_public_keys_payload_iter() {
        if f.is_err() {
            result.push_error(create_verification_error!(
                format!(
                    "control_component_public_keys_payload.{} has wrong format",
                    i
                ),
                f.unwrap_err()
            ))
        } else {
            test_encryption_group(
                &f.unwrap().encryption_group,
                &eg,
                &format!("control_component_public_keys_payload.{}", i),
                result,
            )
        }
    }
    for vcs in setup_dir.vcs_directories().iter() {
        test_encryption_group_for_vcs_dir(vcs, &eg, result);
    }
}

#[cfg(test)]
mod test {
    use num_bigint::BigUint;

    use super::{
        super::super::super::{verification::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::file_structure::{mock::MockVerificationDirectory, VerificationDirectory};
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    fn get_mock_verifier_dir() -> MockVerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        MockVerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_wrong() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_mock_verifier_dir();
        fn_verification(&mock_dir, &mut result);
        assert!(result.is_ok().unwrap());
        let mut eec = mock_dir
            .unwrap_setup()
            .election_event_context_payload()
            .unwrap();
        eec.encryption_group.p = BigUint::from(1234usize);
        mock_dir
            .unwrap_setup_mut()
            .mock_election_event_context_payload(&Ok(&eec));
        fn_verification(&mock_dir, &mut result);
        println!("{:?}", result.failures());
        assert!(result.has_failures().unwrap());
    }
}
