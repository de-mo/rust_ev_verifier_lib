use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::{Verification, VerificationMetaData, VerificationResult},
    VerificationCategory, VerificationPeriod,
};
use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::{setup_directory::VCSDirectory, VerificationDirectory},
};

pub(super) fn get_verification() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "308".to_owned(),
            algorithm: "3.09".to_owned(),
            name: "VerifyElectionEventIdConsistency".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Consistency,
        },
        fn_verification,
    )
}

fn test_election_event_id(
    ee_id: &String,
    expected: &String,
    name: &str,
    result: &mut VerificationResult,
) {
    if ee_id != expected {
        result.push_failure(create_verification_failure!(format!(
            "Election Event ID not equal in {}",
            name
        )));
    }
}

fn test_ee_id_for_vcs_dir(dir: &VCSDirectory, expected: &String, result: &mut VerificationResult) {
    match dir.setup_component_tally_data_payload() {
        Ok(p) => test_election_event_id(
            &p.election_event_id,
            &expected,
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
                test_election_event_id(
                    &p.election_event_id,
                    &expected,
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
            test_election_event_id(
                &f.unwrap().election_event_id,
                &expected,
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

fn fn_verification(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    let ee_id = match setup_dir.election_event_context_payload() {
        Ok(o) => o.election_event_context.election_event_id,
        Err(e) => {
            result.push_error(create_verification_error!(
                "Cannot extract election_event_context_payload",
                e
            ));
            return;
        }
    };
    match setup_dir.setup_component_public_keys_payload() {
        Ok(p) => test_election_event_id(
            &p.election_event_id,
            &ee_id,
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
            test_election_event_id(
                &f.unwrap().election_event_id,
                &ee_id,
                &format!("control_component_public_keys_payload.{}", i),
                result,
            )
        }
    }
    for vcs in setup_dir.vcs_directories_iter() {
        test_ee_id_for_vcs_dir(vcs, &ee_id, result);
    }
}

#[cfg(test)]
mod test {
    use super::super::super::super::verification::VerificationResultTrait;
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
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
