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
    consts::CONTROL_COMPONENT_ID_LIST,
    file_structure::{ContextDirectoryTrait, VerificationDirectoryTrait},
};
use std::collections::HashSet;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let mut list_node_id = vec![];
    for (j, cc_bb) in context_dir.control_component_public_keys_payload_iter() {
        match cc_bb {
            Ok(payload) => {
                list_node_id.push(payload.control_component_public_keys.node_id);
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!("Error reading control_component_public_keys_payload.{}", j),
            )),
        }
    }
    result.append_with_context(
        &verifiy_one_to_for(list_node_id.as_slice()),
        "control_component_public_keys_payload",
    );
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        config::test::{
            CONFIG_TEST, get_test_verifier_mock_setup_dir,
            get_test_verifier_setup_dir as get_verifier_dir, test_data_path,
        },
        consts::{NUMBER_CONTROL_COMPONENTS, test::MIXED_CONTROL_COMPONENT_ID_LIST},
        file_structure::VerificationDirectory,
        verification::VerificationPeriod,
    };

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn change_node_id() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    d.control_component_public_keys.node_id =
                        MIXED_CONTROL_COMPONENT_ID_LIST[j - 1];
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "j={}", j);
            assert!(result.has_failures(), "j={}", j);
        }
    }

    #[test]
    fn remove_node_id() {
        for j in 1..=NUMBER_CONTROL_COMPONENTS {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload_as_deleted(j);
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "j={}", j);
            assert!(result.has_failures(), "j={}", j);
        }
    }

    #[test]
    fn tests_in_directories() {
        let test_dir_path = test_data_path().join("verification_0302");
        for p in test_dir_path
            .read_dir()
            .unwrap()
            .map(|f| f.unwrap().path())
            .filter(|f| f.is_dir())
        {
            let dir = VerificationDirectory::new(&VerificationPeriod::Setup, &p);
            let mut result = VerificationResult::new();
            fn_verification(&dir, &CONFIG_TEST, &mut result);
            assert!(
                !result.has_errors(),
                "path={}",
                p.file_name().unwrap().to_str().unwrap()
            );
            assert!(
                result.has_failures(),
                "path={}",
                p.file_name().unwrap().to_str().unwrap()
            );
        }
    }
}
