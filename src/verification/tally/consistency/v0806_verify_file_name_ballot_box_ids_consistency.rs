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

use std::collections::HashSet;

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{ContextDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait},
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let tally_dir = dir.unwrap_tally();

    let payload = match context_dir.election_event_context_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    let ee_context = &payload.election_event_context;

    let bb_ids = ee_context.bb_ids();
    let bb_dir_names = tally_dir.bb_directory_names();
    let hs_bb_dir_names = bb_dir_names
        .iter()
        .map(|e| e.as_str())
        .collect::<HashSet<_>>();

    bb_ids.iter().for_each(|id| if !hs_bb_dir_names.contains(id) {
        result.push(VerificationEvent::new_failure(&format!(
            "The ballot box id {} from election_event_context_payload is not a ballot box directory",
            id
        )))
    });

    hs_bb_dir_names.iter().for_each(|name| if !bb_ids.contains(name) {
        result.push(VerificationEvent::new_failure(&format!(
            "the ballot box directory {} is not in the list of ballot box ids from election_event_context_payload",
            name
        )))
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        CONFIG_TEST, get_test_verifier_mock_tally_dir,
        get_test_verifier_tally_dir as get_verifier_dir,
    };

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for r in result.errors_to_string() {
                println!("{:?}", r)
            }
            for r in result.failures_to_string() {
                println!("{:?}", r)
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_bb() {
        let mut dir = get_test_verifier_mock_tally_dir();
        dir.context_mut().mock_election_event_context_payload(|d| {
            let mut context = d.election_event_context.verification_card_set_contexts[0].clone();
            context.ballot_box_id = "new-bb-id".to_string();
            d.election_event_context
                .verification_card_set_contexts
                .push(context);
        });
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
        assert!(!result.has_errors());
    }

    #[test]
    fn change_context() {
        let dir = get_verifier_dir();
        let nb_vcs = dir.context().vcs_directories().len();
        for i in 0..nb_vcs {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir
                .context_mut()
                .mock_election_event_context_payload(|d| {
                    d.election_event_context.verification_card_set_contexts[i].ballot_box_id =
                        "modified-bb_id".to_string();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at vcs {}", i);
            assert!(result.has_failures(), "Failed at VCS {}", i);
        }
    }

    #[test]
    fn test_remove_bb() {
        let mut dir = get_test_verifier_mock_tally_dir();
        dir.context_mut().mock_election_event_context_payload(|d| {
            d.election_event_context
                .verification_card_set_contexts
                .pop();
        });
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
        assert!(!result.has_errors());
    }
}
