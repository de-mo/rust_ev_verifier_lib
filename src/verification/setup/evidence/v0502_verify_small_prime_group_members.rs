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

pub(super) fn fn_0502_verify_small_prime_group_members<D: VerificationDirectoryTrait>(
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
    let primes = match ee_context.encryption_group.get_small_prime_group_members(
        VerifierConfig::maximum_number_of_supported_voting_options_n_sup(),
    ) {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Error getting small prime group members"),
            );
            return;
        }
    };
    if ee_context.small_primes.len() != primes.len() {
        result.push(VerificationEvent::new_failure(&format!(
            "length of primes not the same: calculated: {} / expected {}",
            primes.len(),
            ee_context.small_primes.len()
        )))
    }
    if let Some(i) = ee_context
        .small_primes
        .iter()
        .zip(primes.iter())
        .position(|(a, b)| a != b)
    {
        result.push(
            VerificationEvent::new_failure(
                &format!(
                    "Small prime group members are not the same. First error at position {}: calculated {} / expected {}",
                    i + 1,
                    primes[i],
                    ee_context.small_primes[i]
                )
            )
        )
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
    fn test_0502_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0502_verify_small_prime_group_members(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{e:?}");
            }
            for f in result.failures() {
                println!("{f:?}");
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_wrong_p() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.encryption_group
                    .set_p(&(d.encryption_group.p() + Integer::from(2u32)));
            });
        fn_0502_verify_small_prime_group_members(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }

    #[test]
    fn test_small_primes_changed() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.small_primes[1] = 17usize;
            });
        fn_0502_verify_small_prime_group_members(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }

    #[test]
    fn test_small_primes_deleted() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.small_primes = d.small_primes[10..d.small_primes.len() - 1].to_vec();
            });
        fn_0502_verify_small_prime_group_members(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }
}
