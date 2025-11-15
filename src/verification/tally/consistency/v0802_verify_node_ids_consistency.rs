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
    consts::CONTROL_COMPONENT_ID_LIST,
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

fn verifiy_one_to_for(list: &[usize]) -> VerificationResult {
    let mut result = VerificationResult::new();
    if list.iter().collect::<HashSet<_>>()
        != CONTROL_COMPONENT_ID_LIST.iter().collect::<HashSet<_>>()
    {
        result.push(VerificationEvent::new_failure(&format!(
            "The list of node ids (={:?}) does not correspond to the expected list (={:?})",
            list, CONTROL_COMPONENT_ID_LIST
        )))
    }
    result
}

fn verify_for_bb_directory<B: BBDirectoryTrait>(bb_dir: &B) -> VerificationResult {
    let mut result = VerificationResult::new();

    let bb_name = bb_dir.name();

    let mut list_node_id = vec![];
    for (j, cc_bb) in bb_dir.control_component_ballot_box_payload_iter() {
        match cc_bb {
            Ok(payload) => {
                list_node_id.push(payload.node_id);
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "Error reading {}/control_component_ballot_box_payload.{}",
                    bb_name, j
                ),
            )),
        }
    }
    result.append_with_context(
        &verifiy_one_to_for(list_node_id.as_slice()),
        format!("{}/control_component_ballot_box_payload", bb_name),
    );

    let mut list_node_id = vec![];
    for (j, cc_bb) in bb_dir.control_component_shuffle_payload_iter() {
        match cc_bb {
            Ok(payload) => {
                list_node_id.push(payload.node_id);
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "Error reading {}/control_component_shuffle_payload.{}",
                    bb_name, j
                ),
            )),
        }
    }
    result.append_with_context(
        &verifiy_one_to_for(list_node_id.as_slice()),
        format!("{}/control_component_shuffle_payload", bb_name),
    );

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
        consts::{NUMBER_CONTROL_COMPONENTS, test::MIXED_CONTROL_COMPONENT_ID_LIST},
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
    fn test_change_node_id_cc_bb() {
        let nb = get_test_verifier_mock_tally_dir()
            .unwrap_tally()
            .bb_directories()
            .len();
        for i in 0..nb {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_ballot_box_payload(j, |d| {
                        d.node_id = MIXED_CONTROL_COMPONENT_ID_LIST[j - 1];
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "j={}, folder {i}", j);
                assert!(result.has_failures(), "j={}, folder {i}", j);
            }
        }
    }

    #[test]
    fn test_change_node_id_cc_shuffle() {
        let nb = get_test_verifier_mock_tally_dir()
            .unwrap_tally()
            .bb_directories()
            .len();
        for i in 0..nb {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_shuffle_payload(j, |d| {
                        let new_j = match j {
                            1 => 2,
                            2 => 3,
                            3 => 4,
                            4 => 1,
                            _ => unreachable!(),
                        };
                        d.node_id = new_j;
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors(), "j={}, folder {i}", j);
                assert!(result.has_failures(), "j={}, folder {i}", j);
            }
        }
    }
}
