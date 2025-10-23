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
    file_structure::{ContextDirectoryTrait, VerificationDirectoryTrait},
};

const LIST_CC_NUMBER: &[usize] = &[1, 2, 3, 4];

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    result.append(&mut verify_cc_pk_payload(context_dir));
}

fn verifiy_one_to_for(list: &[usize]) -> VerificationResult {
    let mut result = VerificationResult::new();
    if list.iter().collect::<HashSet<_>>() != LIST_CC_NUMBER.iter().collect::<HashSet<_>>() {
        result.push(VerificationEvent::new_failure(&format!(
            "The list of node ids (={:?}) does not correspond to the expected list (={:?})",
            list, LIST_CC_NUMBER
        )))
    }
    result
}

fn verify_cc_pk_payload<C: ContextDirectoryTrait>(dir: &C) -> VerificationResult {
    let mut result = verifiy_one_to_for(
        dir.control_component_public_keys_payload_group()
            .get_numbers()
            .as_slice(),
    )
    .clone_add_context("context/conntrolComponentPubllicKeysPayload.{}.json");
    for (i, payload_res) in dir.control_component_public_keys_payload_iter() {
        match payload_res {
            Ok(paylod) => {
                let node_id = paylod.control_component_public_keys.node_id;
                if node_id != i {
                    result.push(VerificationEvent::new_failure(&format!(
                        "The node_id (={}) in the file does not correspond to the nr (={}) of the file",
                        node_id, i
                    )))
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&format!(
                "Cannot open conntrolComponentPubllicKeysPayload.{}.json: {}",
                i, e
            ))),
        }
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
    fn test_change_node_id() {
        for j in 1..=4 {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_control_component_public_keys_payload(j, |d| {
                    let new_j = match j {
                        1 => 2,
                        2 => 3,
                        3 => 4,
                        4 => 1,
                        _ => unreachable!(),
                    };
                    d.control_component_public_keys.node_id = new_j;
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "j={}", j);
            assert!(result.has_failures(), "j={}", j);
        }
    }

    #[test]
    fn test_remove_file() {
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
