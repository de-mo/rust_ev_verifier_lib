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
        TallyDirectoryTrait, VerificationDirectoryTrait, tally_directory::BBDirectoryTrait,
    },
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();

    for bb_dir in tally_dir.bb_directories().iter() {
        result.append_with_context(
            &verify_for_bb_directory(bb_dir),
            format!("Ballot box directory {}", bb_dir.name()),
        );
    }
}

fn verify_for_bb_directory<B: BBDirectoryTrait>(bb_dir: &B) -> VerificationResult {
    let mut result = VerificationResult::new();

    let bb_id = bb_dir.name();

    for (i, cc_bb_payload_res) in bb_dir.control_component_ballot_box_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => {
                if p.ballot_box_id != bb_id {
                    result.push(VerificationEvent::new_failure(&format!(
                    "bb_id (={}) in {}/control_component_ballot_box_payload_{} is not the same than the directory",
                    &p.ballot_box_id, bb_id, i
                )));
                }
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "{}/control_component_ballot_box_payload_{} cannot be read",
                    bb_id, i
                ),
            )),
        }
    }

    for (i, cc_bb_payload_res) in bb_dir.control_component_shuffle_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => {
                if p.ballot_box_id != bb_id {
                    result.push(VerificationEvent::new_failure(&format!(
                    "bb_id (={}) in {}/control_component_shuffle_payload_{} is not the same than the directory",
                    &p.ballot_box_id, bb_id, i
                )));
                }
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "{}/control_component_shuffle_payload_{} cannot be read",
                    bb_id, i
                ),
            )),
        }
    }

    match bb_dir.tally_component_shuffle_payload() {
        Ok(p) => {
            if p.ballot_box_id != bb_id {
                result.push(VerificationEvent::new_failure(&format!(
                "bb_id (={}) in {}/tally_component_shuffle_payload is not the same than the directory",
                &p.ballot_box_id, bb_id
            )));
            }
        }
        Err(e) => result.push(
            VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/tally_component_shuffle_payload cannot be read",
                bb_id
            )),
        ),
    }

    match bb_dir.tally_component_votes_payload() {
        Ok(p) => {
            if p.ballot_box_id != bb_id {
                result.push(VerificationEvent::new_failure(&format!(
                "bb_id (={}) in {}/tally_component_votes_payload is not the same than the directory",
                &p.ballot_box_id, bb_id
            )));
            }
        }
        Err(e) => result.push(
            VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/tally_component_votes_payload cannot be read",
                bb_id
            )),
        ),
    }

    result
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
    fn change_in_tally_component_votes() {
        let dir = get_verifier_dir();
        for bb in dir.unwrap_tally().bb_directories().iter() {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir
                .unwrap_tally_mut()
                .bb_directory_mut(&bb.name())
                .unwrap()
                .mock_tally_component_votes_payload(|d| {
                    d.ballot_box_id = "modified-bb-id".to_string();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at bb {}", bb.name());
            assert!(result.has_failures(), "Failed at bb {}", bb.name());
        }
    }

    #[test]
    fn change_in_tally_component_shuffle() {
        let dir = get_verifier_dir();
        for bb in dir.unwrap_tally().bb_directories().iter() {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir
                .unwrap_tally_mut()
                .bb_directory_mut(&bb.name())
                .unwrap()
                .mock_tally_component_shuffle_payload(|d| {
                    d.ballot_box_id = "modified-ballot_box_id".to_string();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at bb {}", bb.name());
            assert!(result.has_failures(), "Failed at bb {}", bb.name());
        }
    }

    #[test]
    fn change_in_cc_bb() {
        let dir = get_verifier_dir();
        for bb in dir.unwrap_tally().bb_directories().iter() {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir
                    .unwrap_tally_mut()
                    .bb_directory_mut(&bb.name())
                    .unwrap()
                    .mock_control_component_ballot_box_payload(j, |d| {
                        d.ballot_box_id = "modified-ballot_box_id".to_string();
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

    #[test]
    fn change_in_cc_shuffle() {
        let dir = get_verifier_dir();
        for bb in dir.unwrap_tally().bb_directories().iter() {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir
                    .unwrap_tally_mut()
                    .bb_directory_mut(&bb.name())
                    .unwrap()
                    .mock_control_component_shuffle_payload(j, |d| {
                        d.ballot_box_id = "modified-ballot_box_id".to_string();
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
