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
    data_structures::{VerifierDataDecode, VerifierDataToTypeTrait},
    file_structure::{
        file_group::FileGroup,
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};

fn verify_uninterrupted_monotonic_sequence<
    D: VerifierDataDecode + VerifierDataToTypeTrait + Clone,
>(
    fg: &FileGroup<D>,
    result: &mut VerificationResult,
    dir: &String,
) {
    let mut numbers = fg.get_numbers().clone();
    numbers.sort();
    if !fg.has_elements() && numbers[0] + numbers[numbers.len() - 1] == numbers.len() {
        result.push(VerificationEvent::new_failure(&format!(
            "The sequence is not uniterrupted for files {} in directory {}",
            fg.get_file_name(),
            dir
        )))
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    for vcs in setup_dir.vcs_directories() {
        verify_uninterrupted_monotonic_sequence(
            vcs.setup_component_verification_data_payload_group(),
            result,
            &vcs.name(),
        );
        verify_uninterrupted_monotonic_sequence(
            vcs.control_component_code_shares_payload_group(),
            result,
            &vcs.name(),
        );
        for (i, elt) in vcs.setup_component_verification_data_payload_iter() {
            match elt {
                Ok(p) => {
                    if p.chunk_id != i {
                        result.push(VerificationEvent::new_failure(&format!(
                            "The chunkID nr {} does not matches the chunkID in the file name in {} for setup_component_verification_data_payload",
                            i,
                            vcs.name()
                        )))
                    }
                }
                Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "Error getting setup_component_verification_data_payload for chunk {} in {}",
                    i,
                    vcs.name()
                ))),
            }
        }
        for (i, elt) in vcs.control_component_code_shares_payload_iter() {
            match elt {
                Ok(p) => {
                    for (j, e) in p.0.iter().enumerate() {
                        if e.chunk_id != i {
                            result.push(VerificationEvent::new_failure(&format!(
                            "The chunkID nr {} does not matches the chunkID in the file name in {} for control_component_code_shares_payload at pos {}",
                            i,
                            vcs.name(), j
                        )))
                        }
                    }
                }
                Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "Error getting control_component_code_shares_payload for chunk {} in {}",
                    i,
                    vcs.name()
                ))),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
