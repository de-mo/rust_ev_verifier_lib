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
    let setup_pk_paylod = match context_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot extract setup_component_public_keys_payload"),
            );
            return;
        }
    };

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

    let setup_ccr_pk = &setup_pk_paylod
        .setup_component_public_keys
        .choice_return_codes_encryption_public_key;

    let cc_ccr_pk = cc_pk_payloads
        .iter()
        .map(|cc_pk_payload| {
            cc_pk_payload
                .control_component_public_keys
                .ccrj_choice_return_codes_encryption_public_key
                .as_slice()
        })
        .collect::<Vec<_>>();

    for (j, cc_pk_j) in cc_ccr_pk.iter().enumerate() {
        if cc_pk_j.len() != setup_ccr_pk.len() {
            {
                result.push(VerificationEvent::new_failure(&format!(
                "The number of ccr election public keys in control component public key {} ({}) does not match the number of ccr public keys in setup ({})",
                j,
                cc_pk_j.len(),
                setup_ccr_pk.len()
            )));
            }
        }
    }

    // cannot continue if sizes are not the same
    if result.has_failures() {
        return;
    }

    for (i, ccr) in setup_ccr_pk.iter().enumerate() {
        let product_ccr = cc_ccr_pk
            .iter()
            .map(|e| e.get(i).unwrap())
            .fold(Integer::one().clone(), |acc, x| acc.mod_multiply(x, eg.p()));
        if &product_ccr != ccr {
            result.push(VerificationEvent::new_failure(&format!(
                "The ccr at position {} is not the product of the cc ccr",
                i
            )));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        config::test::{
            CONFIG_TEST, get_test_verifier_mock_setup_dir,
            get_test_verifier_setup_dir as get_verifier_dir,
        },
        consts::NUMBER_CONTROL_COMPONENTS,
    };

    fn nb_pk_ccr() -> usize {
        get_verifier_dir()
            .context()
            .setup_component_public_keys_payload()
            .unwrap()
            .setup_component_public_keys
            .choice_return_codes_encryption_public_key
            .len()
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn change_pk_ccr() {
        for i in 0..nb_pk_ccr() {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_setup_component_public_keys_payload(|d| {
                    d.setup_component_public_keys
                        .choice_return_codes_encryption_public_key[i] = Integer::from(111u32)
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }

    #[test]
    fn add_pk_ccr() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_setup_component_public_keys_payload(|d| {
                d.setup_component_public_keys
                    .choice_return_codes_encryption_public_key
                    .push(Integer::from(111u32));
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn remove_pk_ccr() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_setup_component_public_keys_payload(|d| {
                d.setup_component_public_keys
                    .choice_return_codes_encryption_public_key
                    .pop();
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn change_pk_ccr_j() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
            for i in 0..nb_pk_ccr() {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_setup_dir();
                mock_dir
                    .context_mut()
                    .mock_control_component_public_keys_payload(j, |d| {
                        d.control_component_public_keys
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
    fn add_pk_ccr_j() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys
                        .ccrj_choice_return_codes_encryption_public_key
                        .push(Integer::from(111u32));
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }

    #[test]
    fn remove_pk_ccr_j() {
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
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }
}
