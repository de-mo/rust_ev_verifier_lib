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
    let payload = match context_dir.election_event_context_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot extract election_event_context_payload"),
            );
            return;
        }
    };
    let vcs_contexts = &payload
        .election_event_context
        .verification_card_set_contexts;
    let total_voter = match context_dir.election_event_configuration() {
        Ok(o) => match o.get_data() {
            Ok(o) => o.register.len(),
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e)
                        .add_context("Cannot extract election_event_configuration"),
                );
                return;
            }
        },
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot read election_event_context_payload"),
            );
            return;
        }
    };
    if total_voter
        != vcs_contexts
            .iter()
            .map(|e| e.number_of_eligible_voters)
            .sum::<usize>()
    {
        result.push(VerificationEvent::new_failure(&format!(
            "The sum of voting cards is not the same as total voters {}",
            total_voter
        )))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{CONFIG_TEST, get_test_verifier_setup_dir as get_verifier_dir};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
