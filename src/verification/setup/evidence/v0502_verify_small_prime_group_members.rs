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
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};

pub(super) fn fn_0502_verify_small_prime_group_members<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let eg = match context_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    let primes = match eg.encryption_group.get_small_prime_group_members(
        VerifierConfig::maximum_number_of_supported_voting_options_n_sup(),
    ) {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Error getting small prime group members"),
            );
            return;
        }
    };
    if eg.small_primes.len() != primes.len() {
        result.push(VerificationEvent::new_failure(&format!(
            "length of primes not the same: calculated: {} / expected {}",
            primes.len(),
            eg.small_primes.len()
        )))
    } else if eg.small_primes != primes {
        let mut i = 0usize;
        while eg.small_primes[i] == primes[i] {
            i += 1;
        }
        result.push(
            VerificationEvent::new_failure(
                &format!(
                    "Small prime group members are not the same. First error at position {}: calculated {} / expected {}",
                    i + 1,
                    primes[i],
                    eg.small_primes[i]
                )
            )
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_0502_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0502_verify_small_prime_group_members(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }
}
