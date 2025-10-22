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
    file_structure::{VerificationDirectoryTrait, context_directory::ContextDirectoryTrait},
};
use rust_ev_system_library::preliminaries::{
    GetHashElectionEventContextContext, VerifyKeyGenerationSchnorrProofsInput,
    VerifyKeyGenerationSchnorrProofsOuput,
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let ee_context = match context_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    let setup_cc_ppk_payload = match context_dir.setup_component_public_keys_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("setup_component_public_keys_payload cannot be read"),
            );
            return;
        }
    };

    let get_hash_election_event_context =
        GetHashElectionEventContextContext::from(ee_context.as_ref());

    // Prepare inputs
    let pk_ccr = setup_cc_ppk_payload
        .setup_component_public_keys
        .combined_control_component_public_keys
        .iter()
        .map(|cc| {
            cc.ccrj_choice_return_codes_encryption_public_key
                .iter()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let pi_pkccr = setup_cc_ppk_payload
        .setup_component_public_keys
        .combined_control_component_public_keys
        .iter()
        .map(|cc| {
            cc.ccrj_schnorr_proofs
                .iter()
                .map(|p| p.as_tuple())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let el_pk = setup_cc_ppk_payload
        .setup_component_public_keys
        .combined_control_component_public_keys
        .iter()
        .map(|cc| cc.ccmj_election_public_key.iter().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let pi_elpk = setup_cc_ppk_payload
        .setup_component_public_keys
        .combined_control_component_public_keys
        .iter()
        .map(|cc| {
            cc.ccmj_schnorr_proofs
                .iter()
                .map(|p| p.as_tuple())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let eb_pk = setup_cc_ppk_payload
        .setup_component_public_keys
        .electoral_board_public_key
        .iter()
        .collect::<Vec<_>>();
    let pi_eb = setup_cc_ppk_payload
        .setup_component_public_keys
        .electoral_board_schnorr_proofs
        .iter()
        .map(|p| p.as_tuple())
        .collect::<Vec<_>>();
    let verify_key_generation_schnorr_proofs_input = VerifyKeyGenerationSchnorrProofsInput {
        pk_ccr: pk_ccr.as_slice(),
        pi_pkccr: pi_pkccr.as_slice(),
        el_pk: el_pk.as_slice(),
        pi_elpk: pi_elpk.as_slice(),
        eb_pk: &eb_pk,
        pi_eb: &pi_eb,
    };

    let verif_schnorr_key_generation =
        VerifyKeyGenerationSchnorrProofsOuput::verify_key_generation_schnorr_proofs(
            &get_hash_election_event_context,
            &verify_key_generation_schnorr_proofs_input,
        );

    result.extend(
        verif_schnorr_key_generation
            .errors
            .iter()
            .map(VerificationEvent::new_error)
            .chain(
                verif_schnorr_key_generation
                    .verif_schnorr_ccm
                    .iter()
                    .map(|e| VerificationEvent::new_failure(e).add_context("verif_schnorr_ccm")),
            )
            .chain(
                verif_schnorr_key_generation
                    .verif_schnorr_ccr
                    .iter()
                    .map(|e| VerificationEvent::new_failure(e).add_context("verif_schnorr_ccr")),
            )
            .chain(
                verif_schnorr_key_generation
                    .verif_schnorr_eb
                    .iter()
                    .map(|e| VerificationEvent::new_failure(e).add_context("verif_schnorr_eb")),
            ),
    );
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
        assert!(
            result.is_ok(),
            "errors: {:?} \n failures: {:?}",
            result.errors(),
            result.failures()
        );
    }

    #[test]
    fn change_ccr_pk() {
        for j in 0..4 {
            for i in 0..2 {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        d.setup_component_public_keys
                            .combined_control_component_public_keys
                            .get_mut(j)
                            .unwrap()
                            .ccrj_choice_return_codes_encryption_public_key[i] =
                            Integer::from(111u32)
                    });

                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
            }
        }
    }

    #[test]
    fn change_ccr_pi() {
        for j in 0..4 {
            for i in 0..2 {
                // e
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        d.setup_component_public_keys
                            .combined_control_component_public_keys[j]
                            .ccrj_schnorr_proofs[i]
                            .e = Integer::from(111u32)
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
                // z
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        d.setup_component_public_keys
                            .combined_control_component_public_keys[j]
                            .ccrj_schnorr_proofs[i]
                            .z = Integer::from(111u32)
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
            }
        }
    }

    #[test]
    fn change_ccm_pk() {
        for j in 0..4 {
            for i in 0..2 {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        d.setup_component_public_keys
                            .combined_control_component_public_keys
                            .get_mut(j)
                            .unwrap()
                            .ccmj_election_public_key[i] = Integer::from(111u32)
                    });

                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
            }
        }
    }

    #[test]
    fn change_ccm_pi() {
        for j in 0..4 {
            for i in 0..2 {
                // e
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        d.setup_component_public_keys
                            .combined_control_component_public_keys[j]
                            .ccmj_schnorr_proofs[i]
                            .e = Integer::from(111u32)
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
                // z
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_setup_component_public_keys_payload(|d| {
                        d.setup_component_public_keys
                            .combined_control_component_public_keys[j]
                            .ccmj_schnorr_proofs[i]
                            .z = Integer::from(111u32)
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
            }
        }
    }

    #[test]
    fn change_eb_pk() {
        for i in 0..2 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    d.setup_component_public_keys.electoral_board_public_key[i] =
                        Integer::from(111u32)
                });

            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }

    #[test]
    fn change_eb_pi() {
        for i in 0..2 {
            // e
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    d.setup_component_public_keys.electoral_board_schnorr_proofs[i].e =
                        Integer::from(111u32)
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
            // z
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    d.setup_component_public_keys.electoral_board_schnorr_proofs[i].z =
                        Integer::from(111u32)
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }
}
