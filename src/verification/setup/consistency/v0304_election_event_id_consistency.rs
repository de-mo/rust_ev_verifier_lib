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
        VerificationDirectoryTrait,
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
    },
};

fn test_election_event_id(ee_id: &String, expected: &String) -> VerificationResult {
    let mut result = VerificationResult::new();
    if ee_id != expected {
        result.push(VerificationEvent::new_failure(&format!(
            "Election Event ID {} not equal to {}",
            ee_id, expected
        )));
    }
    result
}

fn test_ee_id_for_context_vcs_dir<V: ContextVCSDirectoryTrait>(
    dir: &V,
    expected: &String,
    result: &mut VerificationResult,
) {
    match dir.setup_component_tally_data_payload() {
        Ok(p) => result.append_with_context(
            &test_election_event_id(&p.election_event_id, expected),
            format!("{}/setup_component_tally_data_payload", dir.name()),
        ),
        Err(e) => result.push(
            VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/setup_component_tally_data_payload has wrong format",
                dir.name()
            )),
        ),
    }
}

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
    let ee_id = &context.as_ref().election_event_context.election_event_id;
    match context_dir.setup_component_public_keys_payload() {
        Ok(p) => result.append_with_context(
            &test_election_event_id(&p.election_event_id, ee_id),
            "setup_component_public_keys_payload",
        ),
        Err(e) => result.push(
            VerificationEvent::new_error_from_error(&e)
                .add_context("election_event_context_payload has wrong format"),
        ),
    }
    for (i, f) in context_dir.control_component_public_keys_payload_iter() {
        match f {
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "control_component_public_keys_payload.{} has wrong format",
                    i
                ),
            )),
            Ok(cc) => result.append_with_context(
                &test_election_event_id(&cc.election_event_id, ee_id),
                format!("control_component_public_keys_payload.{}", i),
            ),
        }
    }
    for vcs in context_dir.vcs_directories().iter() {
        test_ee_id_for_context_vcs_dir(vcs, ee_id, result);
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
    fn change_in_context() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.election_event_context.election_event_id =
                    "modified-election-event-id".to_string();
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn change_setup_pk_keys() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_setup_dir();
        mock_dir
            .context_mut()
            .mock_setup_component_public_keys_payload(|d| {
                d.election_event_id = "modified-election-event-id".to_string();
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn change_cc_pk_keys() {
        for j in 1..=4 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.election_event_id = "modified-election-event-id".to_string();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at CC {}", j);
            assert!(result.has_failures(), "Failed at CC {}", j);
        }
    }

    #[test]
    fn change_setup_tally_data() {
        let dir = get_verifier_dir();
        for vcs in dir.context().vcs_directories().iter() {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .vcs_directory_mut(&vcs.name())
                .unwrap()
                .mock_setup_component_tally_data_payload(|d| {
                    d.election_event_id = "modified-election-event-id".to_string();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at vcs {}", vcs.name());
            assert!(result.has_failures(), "Failed at VCS {}", vcs.name());
        }
    }
}
