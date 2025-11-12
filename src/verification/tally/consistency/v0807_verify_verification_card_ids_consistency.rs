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
    file_structure::{
        ContextDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait,
        context_directory::ContextVCSDirectoryTrait, tally_directory::BBDirectoryTrait,
    },
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

    for vcs_dir in context_dir.vcs_directories().iter() {
        let vcs_id = vcs_dir.name();
        let payload = match vcs_dir.setup_component_tally_data_payload() {
            Ok(p) => p,
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e)
                        .add_context("setup_component_tally_data_payload cannot be read"),
                );
                break;
            }
        };
        let hs_vc_ids = payload
            .verification_card_ids
            .iter()
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();
        let bb_id = match ee_context.find_ballot_box_id(vcs_id.as_str()) {
            Some(id) => id,
            None => {
                result.push(VerificationEvent::new_error(&format!(
                    "ballot box id for vcs_id {} not found in setup_component_tally_data_payload",
                    vcs_id
                )));
                break;
            }
        };
        let bb_dir = match tally_dir
            .bb_directories()
            .iter()
            .find(|p| p.name() == bb_id)
        {
            Some(p) => p,
            None => {
                result.push(VerificationEvent::new_error(&format!(
                    "ballot box for bb_id {} not found in the ballot box directories",
                    bb_id
                )));
                break;
            }
        };
        for (i, cc_bb_paylod) in bb_dir.control_component_ballot_box_payload_iter() {
            if let Err(e) = cc_bb_paylod {
                result.push(
                    VerificationEvent::new_error_from_error(&e).add_context(format!(
                        "{}/control_component_ballot_box_payload_{} cannot be read",
                        bb_id, i
                    )),
                );
                break;
            }
            let bb_vc_ids = cc_bb_paylod
                .as_ref()
                .unwrap()
                .confirmed_encrypted_votes
                .iter()
                .map(|v| v.context_ids.verification_card_id.as_str())
                .collect::<Vec<_>>();
            if !bb_vc_ids.iter().all(|id| hs_vc_ids.contains(id)) {
                result.push(VerificationEvent::new_failure(&format!(
                    "The voting card ids in {}/control_component_ballot_box_payload_{} are not a subset of the vc ids in {}/setup_component_tally_data_payload",
                    bb_id, i, vcs_id
                )));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        config::test::{
            CONFIG_TEST, get_test_verifier_mock_tally_dir,
            get_test_verifier_tally_dir as get_verifier_dir,
        },
        consts::NUMBER_CONTROL_COMPONENTS,
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
    fn modify_cc_bb() {
        let dir = get_verifier_dir();
        for bb in dir.unwrap_tally().bb_directories().iter() {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                let len_votes = bb
                    .control_component_ballot_box_payload_iter()
                    .find(|p| p.0 == j)
                    .unwrap()
                    .1
                    .unwrap()
                    .confirmed_encrypted_votes
                    .len();
                for i in 0..len_votes {
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_tally_dir();
                    mock_dir
                        .unwrap_tally_mut()
                        .bb_directory_mut(&bb.name())
                        .unwrap()
                        .mock_control_component_ballot_box_payload(j, |d| {
                            d.confirmed_encrypted_votes[i]
                                .context_ids
                                .verification_card_id = "modified-vc_id".to_string();
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(
                        !result.has_errors(),
                        "Failed at bb {} cc_bb {}",
                        bb.name(),
                        j
                    );
                    assert!(
                        result.has_failures(),
                        "Failed at bb {} cc_bb {}",
                        bb.name(),
                        j
                    );
                }
            }
        }
    }

    #[test]
    fn modify_context() {
        let dir = get_verifier_dir();
        for vcs in dir.context().vcs_directories().iter() {
            let context = dir.context().election_event_context_payload().unwrap();
            let bb_id = context
                .election_event_context
                .find_ballot_box_id(vcs.name().as_str())
                .unwrap();
            if dir
                .unwrap_tally()
                .bb_directories()
                .iter()
                .find(|bb| bb.tally_component_votes_payload().unwrap().ballot_box_id == bb_id)
                .unwrap()
                .control_component_ballot_box_payload_iter()
                .next()
                .unwrap()
                .1
                .unwrap()
                .confirmed_encrypted_votes
                .is_empty()
            {
                continue;
            }
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir
                .context_mut()
                .vcs_directory_mut(&vcs.name())
                .unwrap()
                .mock_setup_component_tally_data_payload(|p| {
                    p.verification_card_ids.clear();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at vcs {}", vcs.name(),);
            assert!(result.has_failures(), "Failed at vcs {}", vcs.name(),);
        }
    }
}
