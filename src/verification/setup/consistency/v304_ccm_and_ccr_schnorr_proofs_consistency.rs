use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    data_structures::{
        setup::control_component_public_keys_payload::ControlComponentPublicKeys,
        VerifierSetupDataTrait,
    },
    error::{create_verifier_error, VerifierError},
    file_structure::{
        setup_directory::{SetupDirectoryTrait, VCSDirectoryTrait},
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use std::iter::zip;

fn validate_ccm_and_ccr_schorr_proofs<V: VCSDirectoryTrait, S: SetupDirectoryTrait<V>>(
    setup_dir: &S,
    setup: &ControlComponentPublicKeys,
    node_id: usize,
    result: &mut VerificationResult,
) {
    let f = setup_dir
        .control_component_public_keys_payload_group()
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
    if setup.ccmj_schnorr_proofs.len() != cc_pk.ccmj_schnorr_proofs.len() {
        result.push_failure(create_verification_failure!(format!("The length of CCM public keys for control component {} are identical from both sources", node_id)));
    } else {
        for (i, (a, b)) in zip(&setup.ccmj_schnorr_proofs, &cc_pk.ccmj_schnorr_proofs).enumerate() {
            if a.e != b.e {
                result.push_failure(create_verification_failure!(format!(
            "The field e for Ccm Schor Proof is not the same at pos {} for control component {}", i,
            node_id
        )));
            }
            if a.z != b.z {
                result.push_failure(create_verification_failure!(format!(
            "The field z for Ccm Schor Proof is not the same at pos {} for control component {}", i,
            node_id
        )));
            }
        }
    }
    if setup.ccmj_schnorr_proofs.len() != cc_pk.ccmj_schnorr_proofs.len() {
        result.push_failure(create_verification_failure!(format!("The length of CCM public keys for control component {} are identical from both sources", node_id)));
    } else {
        for (i, (a, b)) in zip(&setup.ccmj_schnorr_proofs, &cc_pk.ccmj_schnorr_proofs).enumerate() {
            if a.e != b.e {
                result.push_failure(create_verification_failure!(format!(
            "The field e for Ccm Schor Proof is not the same at pos {} for control component {}", i,
            node_id
        )));
            }
            if a.z != b.z {
                result.push_failure(create_verification_failure!(format!(
            "The field z for Ccm Schor Proof is not the same at pos {} for control component {}", i,
            node_id
        )));
            }
        }
    }
}

pub(super) fn fn_verification<
    B: BBDirectoryTrait,
    V: VCSDirectoryTrait,
    S: SetupDirectoryTrait<V>,
    T: TallyDirectoryTrait<B>,
>(
    dir: &dyn VerificationDirectoryTrait<B, V, S, T>,
    result: &mut VerificationResult,
) {
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
        validate_ccm_and_ccr_schorr_proofs(setup_dir, &node, node.node_id as usize, result)
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::super::{verification::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::file_structure::VerificationDirectory;
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
