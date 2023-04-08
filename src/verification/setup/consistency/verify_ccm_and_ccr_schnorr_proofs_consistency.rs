use std::iter::zip;

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

pub(super) fn get_verification_304() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "304".to_owned(),
            nr: "3.05".to_owned(),
            name: "VerifyCcmAndCcrSchnorrProofsConsistency".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Consistency,
        },
        fn_verification_304,
    )
}

fn validate_ccm_and_ccr_schorr_proofs(
    setup_dir: &SetupDirectory,
    setup: &ControlComponentPublicKeys,
    node_id: usize,
) -> Result<Vec<VerificationFailure>, VerificationError> {
    let mut failures: Vec<VerificationFailure> = vec![];
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
    if setup.ccmj_schnorr_proofs.len() != cc_pk.ccmj_schnorr_proofs.len() {
        failures.push(create_verification_failure!(format!("The length of CCM public keys for control component {} are identical from both sources", node_id)));
    } else {
        for (i, (a, b)) in zip(&setup.ccmj_schnorr_proofs, &cc_pk.ccmj_schnorr_proofs).enumerate() {
            if a.e != b.e {
                failures.push(create_verification_failure!(format!(
            "The field e for Ccm Schor Proof is not the same at pos {} for control component {}", i,
            node_id
        )));
            }
            if a.z != b.z {
                failures.push(create_verification_failure!(format!(
            "The field z for Ccm Schor Proof is not the same at pos {} for control component {}", i,
            node_id
        )));
            }
        }
    }
    if setup.ccmj_schnorr_proofs.len() != cc_pk.ccmj_schnorr_proofs.len() {
        failures.push(create_verification_failure!(format!("The length of CCM public keys for control component {} are identical from both sources", node_id)));
    } else {
        for (i, (a, b)) in zip(&setup.ccmj_schnorr_proofs, &cc_pk.ccmj_schnorr_proofs).enumerate() {
            if a.e != b.e {
                failures.push(create_verification_failure!(format!(
            "The field e for Ccm Schor Proof is not the same at pos {} for control component {}", i,
            node_id
        )));
            }
            if a.z != b.z {
                failures.push(create_verification_failure!(format!(
            "The field z for Ccm Schor Proof is not the same at pos {} for control component {}", i,
            node_id
        )));
            }
        }
    }
    Ok(failures)
}

fn fn_verification_304(
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
        match validate_ccm_and_ccr_schorr_proofs(setup_dir, &node, node.node_id as usize) {
            Ok(mut r) => failures.append(&mut r),
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
        let (e, f) = fn_verification_304(&dir);
        assert!(e.is_empty());
        assert!(f.is_empty());
    }
}
