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
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};

fn validate_cc_ccm_pk(
    setup_dir: &SetupDirectory,
    setup: &ControlComponentPublicKeys,
    node_id: usize,
    result: &mut VerificationResult,
) {
    let f = setup_dir
        .control_component_public_keys_payload_group
        .get_file_with_number(node_id);
    let cc_pk = match f
        .get_data()
        .map(|d| Box::new(d.control_component_public_keys_payload().unwrap().clone()))
    {
        Ok(d) => d.control_component_public_keys,
        Err(e) => {
            result.push_error(create_verification_error!(
                format!("Cannot read data from file {}", f.to_str()),
                e
            ));
            return;
        }
    };
    if setup.ccmj_election_public_key.len() != cc_pk.ccmj_election_public_key.len() {
        result.push_failure(create_verification_failure!(format!("The length of CCM public keys for control component {} are identical from both sources", node_id)));
    } else {
        if setup.ccrj_choice_return_codes_encryption_public_key
            != cc_pk.ccrj_choice_return_codes_encryption_public_key
        {
            result.push_failure(create_verification_failure!(format!(
                "The CCM public keys for control component {} are identical from both sources",
                node_id
            )));
        };
    }
}

pub(super) fn fn_verification(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    let sc_pk = match setup_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push_error(create_verification_error!(
                "Cannot extract setup_component_public_keys_payload",
                e
            ));
            return;
        }
    };
    for node in sc_pk
        .setup_component_public_keys
        .combined_control_component_public_keys
    {
        validate_cc_ccm_pk(setup_dir, &node, node.node_id as usize, result)
    }
}

#[cfg(test)]
mod test {
    use crate::verification::VerificationPeriod;

    use super::super::super::super::verification::VerificationResultTrait;
    use super::*;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
