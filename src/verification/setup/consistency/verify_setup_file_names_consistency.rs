use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::file::File,
};

use crate::file_structure::VerificationDirectory;

use super::super::super::{
    error::{
        create_verification_failure, VerificationError, VerificationFailure,
        VerificationFailureType,
    },
    verification::{Verification, VerificationMetaData},
    VerificationCategory, VerificationPeriod,
};

pub(super) fn get_verification_301() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "301".to_owned(),
            nr: "3.02".to_owned(),
            name: "VerifySetupFileNamesConsistency".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Consistency,
        },
        fn_verification_301,
    )
}

fn test_file_exists(file: &File, failure: &mut Vec<VerificationFailure>) {
    if !file.exists() {
        failure.push(create_verification_failure!(format!(
            "File {} does not exist",
            file.to_str()
        )))
    }
}

fn fn_verification_301(
    dir: &VerificationDirectory,
) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut failures: Vec<VerificationFailure> = vec![];
    let setup_dir = dir.unwrap_setup();
    test_file_exists(&setup_dir.encryption_parameters_payload_file, &mut failures);
    test_file_exists(
        &setup_dir.election_event_context_payload_file,
        &mut failures,
    );
    test_file_exists(
        &setup_dir.setup_component_public_keys_payload_file,
        &mut failures,
    );
    let mut cc_group_numbers = setup_dir
        .control_component_public_keys_payload_group
        .get_numbers();
    cc_group_numbers.sort();
    if cc_group_numbers != vec![1, 2, 3, 4] {
        failures.push(create_verification_failure!(format!(
            "controlComponentPublicKeysPayload must have file from 1 to 4. But actually: {:?}",
            cc_group_numbers
        )))
    }
    for (_, f) in setup_dir.control_component_public_keys_payload_group.iter() {
        test_file_exists(&f, &mut failures);
    }
    (vec![], failures)
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
        let (e, f) = fn_verification_301(&dir);
        assert!(e.is_empty());
        assert!(f.is_empty());
    }
}
