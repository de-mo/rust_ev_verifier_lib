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

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();

    let setup_pk_payload = match context_dir.setup_component_public_keys_payload() {
        Ok(d) => d,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot read payload for setup_component_public_keys_payload"),
            );
            return;
        }
    };

    for (j, cc_pk_payload_res) in context_dir.control_component_public_keys_payload_iter() {
        match cc_pk_payload_res {
            Ok(cc_pk_payload) => {
                let node_id = cc_pk_payload.control_component_public_keys.node_id;
                let cc_ccrj_pk = cc_pk_payload
                    .control_component_public_keys
                    .ccrj_choice_return_codes_encryption_public_key
                    .as_slice();
                match setup_pk_payload
                    .setup_component_public_keys
                    .combined_control_component_public_keys
                    .iter()
                    .find(|n| n.node_id == node_id)
                {
                    Some(cc_combined_j) => {
                        let setup_ccrj_pk = cc_combined_j
                            .ccrj_choice_return_codes_encryption_public_key
                            .as_slice();
                        if setup_ccrj_pk.len() != cc_ccrj_pk.len() {
                            result.push(VerificationEvent::new_failure(&format!(
                                "The length of CCR Choice Return Codes encryption public keys for control component {} from control_component_public_keys_payload.{} is not the same as in setup_component_public_keys_payload",
                                node_id, j
                            )));
                        } else {
                            for (i, (setup_ccrj_pk_i, cc_ccrj_pk_i)) in
                                setup_ccrj_pk.iter().zip(cc_ccrj_pk.iter()).enumerate()
                            {
                                if setup_ccrj_pk_i != cc_ccrj_pk_i {
                                    result.push(VerificationEvent::new_failure(&format!(
                                        "A CCR Choice Return Codes encryption public key for control component {} from control_component_public_keys_payload.{} does not match the one in setup_component_public_keys_payload at position {}",
                                        node_id, j, i
                                    )));
                                }
                            }
                        }
                    }
                    None => {
                        result.push(VerificationEvent::new_failure(&format!(
                            "The control component {} from control_component_public_keys_payload.{} is missing from setup_component_public_keys_payload",
                            node_id, j
                        )));
                    }
                }
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!("Cannot read control_component_public_keys_payload.{}", j),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;

    use super::*;
    use crate::{config::test::{
        get_test_verifier_mock_setup_dir, get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST
    }, consts::NUMBER_CONTROL_COMPONENTS};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn change_setup_ccr() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
            let ccr_len = get_verifier_dir()
                .context()
                .setup_component_public_keys_payload()
                .unwrap()
                .setup_component_public_keys
                .combined_control_component_public_keys
                .iter()
                .find(|n| n.node_id == j)
                .unwrap()
                .ccrj_choice_return_codes_encryption_public_key
                .len();
            for i in 0..ccr_len {
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
                        cc_pk.ccrj_choice_return_codes_encryption_public_key[i] =
                            Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
            }
        }
    }

    #[test]
    fn add_setup_ccr() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
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
                    cc_pk
                        .ccrj_choice_return_codes_encryption_public_key
                        .push(Integer::from(111usize));
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn remove_setup_ccr() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
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
                    cc_pk.ccrj_choice_return_codes_encryption_public_key.pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn change_cc_ccr() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
            let ccr_len = get_verifier_dir()
                .context()
                .control_component_public_keys_payload_group()
                .get_file_with_number(j)
                .decode_verifier_data()
                .unwrap()
                .control_component_public_keys
                .ccrj_choice_return_codes_encryption_public_key
                .len();
            for i in 0..ccr_len {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_control_component_public_keys_payload(j, |d| {
                        d.control_component_public_keys
                            .ccrj_choice_return_codes_encryption_public_key[i] =
                            Integer::from(111usize);
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "Failed at CC {} at pos {}", j, i);
                assert!(result.has_failures(), "Failed at CC {} at pos {}", j, i);
            }
        }
    }

    #[test]
    fn add_cc_ccr() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys
                        .ccrj_choice_return_codes_encryption_public_key
                        .push(Integer::from(111usize));
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn remove_cc_ccr() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys
                        .ccrj_choice_return_codes_encryption_public_key
                        .pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }
}
