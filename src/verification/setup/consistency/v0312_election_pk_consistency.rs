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
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ConstantsTrait, OperationsTrait};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let context = match context_dir.election_event_context_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    let eg = &context.as_ref().encryption_group;
    let setup_component_pk_payload = match context_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot extract setup_component_public_keys_payload"),
            );
            return;
        }
    };

    let setup_el_pk = setup_component_pk_payload
        .setup_component_public_keys
        .election_public_key
        .as_slice();
    let expected_keys_len = setup_el_pk.len();

    let setup_eb_pk = setup_component_pk_payload
        .setup_component_public_keys
        .electoral_board_public_key
        .as_slice();

    if setup_eb_pk.len() != expected_keys_len {
        result.push(VerificationEvent::new_failure(&format!(
            "The number of electoral board public keys ({}) does not match the number of election public keys ({})",
            setup_eb_pk.len(),
            expected_keys_len
        )));
    }

    let cc_pk_payloads_res = context_dir
        .control_component_public_keys_payload_iter()
        .map(|(j, cc_pk_payload_res)| match cc_pk_payload_res {
            Ok(cc_pk_payload) => Ok(cc_pk_payload),
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e).add_context(format!(
                        "Cannot extract control_component_public_keys_payload_{j}"
                    )),
                );
                Err(false)
            }
        })
        .collect::<Result<Vec<_>, _>>();

    let cc_pk_payloads = match cc_pk_payloads_res {
        Ok(p) => p,
        Err(_) => return,
    };

    let cc_pk = cc_pk_payloads
        .iter()
        .map(|cc_pk_payload| {
            cc_pk_payload
                .control_component_public_keys
                .ccmj_election_public_key
                .as_slice()
        })
        .collect::<Vec<_>>();

    for (j, cc_pk_j) in cc_pk.iter().enumerate() {
        if cc_pk_j.len() != expected_keys_len {
            {
                result.push(VerificationEvent::new_failure(&format!(
                "The number of ccm election public keys in combined control component public key {} ({}) does not match the number of election public keys ({})",
                j,
                cc_pk_j.len(),
                expected_keys_len
            )));
            }
        }
    }

    // cannot continue if sizes are not the same
    if result.has_failures() {
        return;
    }

    for (i, (el_pk_i, eb_pk_i)) in setup_el_pk.iter().zip(setup_eb_pk.iter()).enumerate() {
        let product_cc_el_pk = cc_pk
            .iter()
            .map(|e| e.get(i).unwrap())
            .fold(Integer::one().clone(), |acc, x| acc.mod_multiply(x, eg.p()));
        let calculated_el_pk = product_cc_el_pk.mod_multiply(eb_pk_i, eg.p());
        if &calculated_el_pk != el_pk_i {
            result.push(VerificationEvent::new_failure(&format!(
                "The election public key EL_pk at {} is correctly combined",
                i
            )));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        CONFIG_TEST, get_test_verifier_mock_setup_dir, get_test_verifier_setup_dir,
    };

    #[test]
    fn test_ok() {
        let dir = get_test_verifier_setup_dir();
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
    fn change_el_pk() {
        for i in 0..2 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    d.setup_component_public_keys.election_public_key[i] = Integer::from(111u32)
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }

    #[test]
    fn change_ccm_el() {
        for j in 1..=4 {
            for i in 0..2 {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_control_component_public_keys_payload(j, |d| {
                        d.control_component_public_keys.ccmj_election_public_key[i] =
                            Integer::from(111u32);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
            }
        }
    }

    #[test]
    fn remove_eb_pk() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_setup_component_public_keys_payload(|d| {
                d.setup_component_public_keys
                    .electoral_board_public_key
                    .pop();
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn remove_el_pk() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_setup_component_public_keys_payload(|d| {
                d.setup_component_public_keys.election_public_key.pop();
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn remove_ccm_el() {
        for j in 1..=4 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys
                        .ccmj_election_public_key
                        .pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }

    #[test]
    fn add_eb_pk() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_setup_component_public_keys_payload(|d| {
                d.setup_component_public_keys
                    .electoral_board_public_key
                    .push(Integer::from(111u32));
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn add_el_pk() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_setup_component_public_keys_payload(|d| {
                d.setup_component_public_keys
                    .election_public_key
                    .push(Integer::from(111u32));
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn add_ccm_el() {
        for j in 1..=4 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys
                        .ccmj_election_public_key
                        .push(Integer::from(111u32));
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }
}
