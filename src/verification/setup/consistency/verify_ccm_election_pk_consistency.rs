use crate::{
    data_structures::{
        setup::control_component_public_keys_payload::ControlComponentPublicKeys, VerifierDataTrait,
    },
    error::{create_verifier_error, VerifierError},
    file_structure::setup_directory::SetupDirectory,
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

pub(super) fn get_verification_303() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "303".to_owned(),
            nr: "3.04".to_owned(),
            name: "VerifyCcmElectionPublicKeyConsistency".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Consistency,
        },
        fn_verification_303,
    )
}

fn validate_cc_ccm_pk(
    setup_dir: &SetupDirectory,
    setup: &ControlComponentPublicKeys,
    node_id: usize,
) -> Result<Option<VerificationFailure>, VerificationError> {
    let f = setup_dir
        .control_component_public_keys_payload_group
        .get_file_with_number(node_id);
    let cc_pk = match f
        .get_data()
        .map(|d| Box::new(d.control_component_public_keys_payload().unwrap().clone()))
    {
        Ok(d) => d.control_component_public_keys,
        Err(e) => {
            return Err(create_verification_error!(
                format!("Cannot read data from file {}", f.to_str()),
                e
            ))
        }
    };
    if setup.ccmj_election_public_key.len() != cc_pk.ccmj_election_public_key.len() {
        return Ok(Some(create_verification_failure!(format!("The length of CCM public keys for control component {} are identical from both sources", node_id))));
    };
    if setup.ccrj_choice_return_codes_encryption_public_key
        != cc_pk.ccrj_choice_return_codes_encryption_public_key
    {
        return Ok(Some(create_verification_failure!(format!(
            "The CCM public keys for control component {} are identical from both sources",
            node_id
        ))));
    };
    Ok(None)
}

fn fn_verification_303(
    dir: &VerificationDirectory,
) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut errors: Vec<VerificationError> = vec![];
    let mut failures: Vec<VerificationFailure> = vec![];
    let setup_dir = dir.unwrap_setup();
    let sc_pk = match setup_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            return (
                vec![create_verification_error!(
                    "Cannot extract setup_component_public_keys_payload",
                    e
                )],
                vec![],
            )
        }
    };
    for node in sc_pk
        .setup_component_public_keys
        .combined_control_component_public_keys
    {
        match validate_cc_ccm_pk(setup_dir, &node, node.node_id as usize) {
            Ok(r) => match r {
                Some(f) => failures.push(f),
                None => (),
            },
            Err(e) => errors.push(e),
        }
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
        let (e, f) = fn_verification_303(&dir);
        assert!(e.is_empty());
        assert!(f.is_empty());
    }
}
