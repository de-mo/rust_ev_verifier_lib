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
    file_structure::{
        context_directory::ContextDirectoryTrait, setup_directory::SetupDirectoryTrait,
        VerificationDirectoryTrait,
    },
};

fn verify_file_name_correct(vcs_ids: &[&str], dir_names: &[String]) -> VerificationResult {
    let mut res = VerificationResult::new();
    let mut vcs_ids_ordered = vcs_ids.to_vec();
    vcs_ids_ordered.sort();
    let mut dir_names_ordered = dir_names.to_vec();
    dir_names_ordered.sort();
    if vcs_ids_ordered != dir_names_ordered {
        res.push(VerificationEvent::new_failure(&format!(
            "The subdirectory [{}] are not equal to the list of voting card set ids [{}]",
            dir_names_ordered.join(","),
            vcs_ids_ordered.join(",")
        )))
    }
    res
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_dir = dir.unwrap_setup();

    let ee_context = match context_dir.election_event_context_payload() {
        Ok(payload) => payload,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot read payload for election_event_context_payload"),
            );
            return;
        }
    };
    let vcs_ids = ee_context.election_event_context.vcs_ids();

    result.append_with_context(
        &verify_file_name_correct(&vcs_ids, &context_dir.vcs_directory_names()),
        "Context directory",
    );

    result.append_with_context(
        &verify_file_name_correct(&vcs_ids, &setup_dir.vcs_directory_names()),
        "Setup directory",
    );
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        get_test_verifier_mock_setup_dir as get_mock_verifier_dir,
        get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST,
    };

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{}", e);
            }
            for f in result.failures() {
                println!("{}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_nok_add_vcs() {
        let mut dir = get_mock_verifier_dir();
        dir.context_mut().mock_election_event_context_payload(|d| {
            let mut context = d.election_event_context.verification_card_set_contexts[0].clone();
            context.verification_card_set_id = "toto".to_string();
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
    fn test_nok_change_vcs_id() {
        let mut dir = get_mock_verifier_dir();
        dir.context_mut().mock_election_event_context_payload(|d| {
            d.election_event_context.verification_card_set_contexts[0].verification_card_set_id =
                "toto".to_string();
        });
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
        assert!(!result.has_errors());
    }

    #[test]
    fn test_nok_change_remove_vcs() {
        let mut dir = get_mock_verifier_dir();
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
