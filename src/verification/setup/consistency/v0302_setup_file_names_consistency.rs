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
        context_directory::ContextDirectoryTrait, file::File, VerificationDirectoryTrait,
    },
};

fn test_file_exists<D: VerifierDataDecode + VerifierDataToTypeTrait>(
    file: &File<D>,
    result: &mut VerificationResult,
) {
    if !file.exists() {
        result.push(VerificationEvent::new_failure(&format!(
            "File {} does not exist",
            file.path_to_str()
        )))
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    test_file_exists(context_dir.election_event_context_payload_file(), result);
    test_file_exists(
        context_dir.setup_component_public_keys_payload_file(),
        result,
    );
    let mut cc_group_numbers = context_dir
        .control_component_public_keys_payload_group()
        .get_numbers()
        .clone();
    cc_group_numbers.sort();
    if cc_group_numbers != vec![1, 2, 3, 4] {
        result.push(VerificationEvent::new_failure(&format!(
            "controlComponentPublicKeysPayload must have file from 1 to 4. But actually: {:?}",
            cc_group_numbers
        )))
    }
    for (_, f) in context_dir
        .control_component_public_keys_payload_group()
        .iter_file()
    {
        test_file_exists(&f, result);
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
