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
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ConstantsTrait, Integer};

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
    let mut p_tilde = vec![];
    for vcsc in ee_context
        .election_event_context
        .verification_card_set_contexts
        .iter()
    {
        p_tilde.extend(
            vcsc.primes_mapping_table
                .p_table
                .iter()
                .map(|e| e.encoded_voting_option),
        );
    }
    p_tilde.sort(); // Sort the primes
    p_tilde.dedup(); // remove duplicates
    let p_prime: Vec<usize> = ee_context
        .small_primes
        .iter()
        .take(p_tilde.len())
        .copied()
        .collect();
    if p_prime != p_tilde {
        result.push(VerificationEvent::new_failure(
            "VerifA: prime group members and encoding voting options are not the same",
        ))
    }
    let verifb = ee_context
        .small_primes
        .iter()
        .take(VerifierConfig::maximum_number_of_supported_voting_options_n_sup() - 1)
        .skip(
            VerifierConfig::maximum_number_of_supported_voting_options_n_sup()
                - VerifierConfig::maximum_supported_number_of_selections_psi_sup()
                - 1,
        )
        .fold(Integer::one().clone(), |acc, e| acc * e);
    if &verifb >= ee_context.encryption_group.p() {
        result.push(VerificationEvent::new_failure(
            "VerifB: The product of the phi last primes (the largest possible encoded vote) must be smaller than p"
        ))
    }
}

#[cfg(test)]
mod test {
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
    fn small_p() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.encryption_group.set_p(&Integer::from(101u32));
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }

    #[test]
    fn change_small_primes() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.small_primes[1] = 17usize;
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }

    #[test]
    fn change_small_primes_tilde() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.election_event_context.verification_card_set_contexts[0]
                    .primes_mapping_table
                    .p_table[1]
                    .encoded_voting_option = 17usize;
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }
}
