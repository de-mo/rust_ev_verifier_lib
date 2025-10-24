// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    data_structures::{
        common_types::SchnorrProof,
        context::control_component_public_keys_payload::ControlComponentPublicKeys,
    },
    file_structure::{VerificationDirectoryTrait, context_directory::ContextDirectoryTrait},
};
use std::iter::zip;

fn validate_schnorr_proofs(
    setup_proofs: &[SchnorrProof],
    cc_proofs: &[SchnorrProof],
) -> VerificationResult {
    let mut result = VerificationResult::new();
    if setup_proofs.len() != cc_proofs.len() {
        result.push(VerificationEvent::new_failure("The lengths of the Schnorr proofs do not match between setup and control component public keys"));
    } else {
        for (i, (setup_proof, cc_proof)) in zip(setup_proofs, cc_proofs).enumerate() {
            if setup_proof.e != cc_proof.e {
                result.push(VerificationEvent::new_failure(&format!(
                    "The field e for Schnorr Proof is not the same at position {}",
                    i
                )));
            }
            if setup_proof.z != cc_proof.z {
                result.push(VerificationEvent::new_failure(&format!(
                    "The field z for Schnorr Proof is not the same at position {}",
                    i
                )));
            }
        }
    }
    result
}

fn validate_ccm_and_ccr_schorr_proofs(
    setup: &ControlComponentPublicKeys,
    cc: &ControlComponentPublicKeys,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    result.append_with_context(
        &validate_schnorr_proofs(&setup.ccmj_schnorr_proofs, &cc.ccmj_schnorr_proofs),
        "Validating CCMJ Schnorr Proofs",
    );
    result.append_with_context(
        &validate_schnorr_proofs(&setup.ccrj_schnorr_proofs, &cc.ccrj_schnorr_proofs),
        "Validating CCRJ Schnorr Proofs",
    );
    result
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_pk_payload = match context_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot extract setup_component_public_keys_payload"),
            );
            return;
        }
    };

    for (j, cc_pk_payload_res) in context_dir.control_component_public_keys_payload_iter() {
        match cc_pk_payload_res {
            Ok(cc_pk_payload) => {
                let node_id = cc_pk_payload.control_component_public_keys.node_id;
                match setup_pk_payload
                    .setup_component_public_keys
                    .combined_control_component_public_keys
                    .iter()
                    .find(|n| n.node_id == node_id)
                {
                    Some(setup_combined_j) => {
                        result.append_with_context(
                            &validate_ccm_and_ccr_schorr_proofs(
                                setup_combined_j,
                                &cc_pk_payload.control_component_public_keys,
                            ),
                            format!(
                                "Validating CCMJ and CCRJ Schnorr Proofs for control component {}",
                                node_id
                            ),
                        );
                    }
                    None => {
                        result.push(VerificationEvent::new_failure(&format!(
                            "The control component {} from control_component_public_keys_payload.{} is missing from setup_component_public_keys_payload",
                            node_id, j
                        )));
                    }
                }
            }
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e).add_context(format!(
                        "Cannot extract control_component_public_keys_payload_{j}"
                    )),
                );
                return;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;

    use super::*;
    use crate::config::test::{
        CONFIG_TEST, get_test_verifier_mock_setup_dir,
        get_test_verifier_setup_dir as get_verifier_dir,
    };

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn change_setup_ccm() {
        for j in 1..=4 {
            let ccm_len = get_verifier_dir()
                .context()
                .setup_component_public_keys_payload()
                .unwrap()
                .setup_component_public_keys
                .combined_control_component_public_keys
                .iter()
                .find(|n| n.node_id == j)
                .unwrap()
                .ccmj_schnorr_proofs
                .len();
            for i in 0..ccm_len {
                // e
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                let mut result = VerificationResult::new();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        let cc_pk = d
                            .setup_component_public_keys
                            .combined_control_component_public_keys
                            .iter_mut()
                            .find(|n| n.node_id == j)
                            .unwrap();
                        cc_pk.ccmj_schnorr_proofs[i].e = Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
                // z
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                let mut result = VerificationResult::new();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        let cc_pk = d
                            .setup_component_public_keys
                            .combined_control_component_public_keys
                            .iter_mut()
                            .find(|n| n.node_id == j)
                            .unwrap();
                        cc_pk.ccmj_schnorr_proofs[i].z = Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
            }
        }
    }

    #[test]
    fn change_setup_ccr() {
        for j in 1..=4 {
            let ccm_len = get_verifier_dir()
                .context()
                .setup_component_public_keys_payload()
                .unwrap()
                .setup_component_public_keys
                .combined_control_component_public_keys
                .iter()
                .find(|n| n.node_id == j)
                .unwrap()
                .ccrj_schnorr_proofs
                .len();
            for i in 0..ccm_len {
                // e
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                let mut result = VerificationResult::new();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        let cc_pk = d
                            .setup_component_public_keys
                            .combined_control_component_public_keys
                            .iter_mut()
                            .find(|n| n.node_id == j)
                            .unwrap();
                        cc_pk.ccrj_schnorr_proofs[i].e = Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
                // z
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                let mut result = VerificationResult::new();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        let cc_pk = d
                            .setup_component_public_keys
                            .combined_control_component_public_keys
                            .iter_mut()
                            .find(|n| n.node_id == j)
                            .unwrap();
                        cc_pk.ccrj_schnorr_proofs[i].z = Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
            }
        }
    }

    #[test]
    fn add_setup_ccmj() {
        for j in 1..=4 {
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            let mut result = VerificationResult::new();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    let cc_pk = d
                        .setup_component_public_keys
                        .combined_control_component_public_keys
                        .iter_mut()
                        .find(|n| n.node_id == j)
                        .unwrap();
                    cc_pk.ccmj_schnorr_proofs.push(SchnorrProof {
                        e: Integer::from(111usize),
                        z: Integer::from(222usize),
                    });
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn add_setup_ccrj() {
        for j in 1..=4 {
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            let mut result = VerificationResult::new();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    let cc_pk = d
                        .setup_component_public_keys
                        .combined_control_component_public_keys
                        .iter_mut()
                        .find(|n| n.node_id == j)
                        .unwrap();
                    cc_pk.ccrj_schnorr_proofs.push(SchnorrProof {
                        e: Integer::from(111usize),
                        z: Integer::from(222usize),
                    });
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn remove_setup_ccm() {
        for j in 1..=4 {
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            let mut result = VerificationResult::new();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    let cc_pk = d
                        .setup_component_public_keys
                        .combined_control_component_public_keys
                        .iter_mut()
                        .find(|n| n.node_id == j)
                        .unwrap();
                    cc_pk.ccmj_schnorr_proofs.pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn remove_setup_ccr() {
        for j in 1..=4 {
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            let mut result = VerificationResult::new();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    let cc_pk = d
                        .setup_component_public_keys
                        .combined_control_component_public_keys
                        .iter_mut()
                        .find(|n| n.node_id == j)
                        .unwrap();
                    cc_pk.ccrj_schnorr_proofs.pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn change_cc_ccm() {
        for j in 1..=4 {
            let ccm_len = get_verifier_dir()
                .context()
                .control_component_public_keys_payload_group()
                .get_file_with_number(j)
                .decode_verifier_data()
                .unwrap()
                .control_component_public_keys
                .ccmj_schnorr_proofs
                .len();
            for i in 0..ccm_len {
                // e
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_control_component_public_keys_payload(j, |d| {
                        d.control_component_public_keys.ccmj_schnorr_proofs[i].e =
                            Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
                // z
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_control_component_public_keys_payload(j, |d| {
                        d.control_component_public_keys.ccmj_schnorr_proofs[i].z =
                            Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
            }
        }
    }

    #[test]
    fn change_cc_ccr() {
        for j in 1..=4 {
            let ccm_len = get_verifier_dir()
                .context()
                .control_component_public_keys_payload_group()
                .get_file_with_number(j)
                .decode_verifier_data()
                .unwrap()
                .control_component_public_keys
                .ccrj_schnorr_proofs
                .len();
            for i in 0..ccm_len {
                // e
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_control_component_public_keys_payload(j, |d| {
                        d.control_component_public_keys.ccrj_schnorr_proofs[i].e =
                            Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
                // z
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_control_component_public_keys_payload(j, |d| {
                        d.control_component_public_keys.ccrj_schnorr_proofs[i].z =
                            Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
            }
        }
    }

    #[test]
    fn add_cc_ccm() {
        for j in 1..=4 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys
                        .ccmj_schnorr_proofs
                        .push(SchnorrProof {
                            e: Integer::from(111usize),
                            z: Integer::from(222usize),
                        });
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn add_cc_ccr() {
        for j in 1..=4 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys
                        .ccrj_schnorr_proofs
                        .push(SchnorrProof {
                            e: Integer::from(111usize),
                            z: Integer::from(222usize),
                        });
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn remove_cc_ccm() {
        for j in 1..=4 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys.ccmj_schnorr_proofs.pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn remove_cc_ccr() {
        for j in 1..=4 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys.ccrj_schnorr_proofs.pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }
}
