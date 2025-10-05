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
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();

    for (i, file) in context_dir
        .control_component_public_keys_payload_group()
        .iter_file()
    {
        match file.decode_verifier_data() {
            Ok(payload) => {
                let node_id = payload.control_component_public_keys.node_id;
                let calculated_path = file
                    .location()
                    .join(format!("controlComponentPublicKeysPayload.{node_id}.json"));
                if calculated_path != file.path() {
                    result.push(VerificationEvent::new_failure(&format!(
                        "The fie has the path {}. Expected: {}",
                        file.path_to_str(),
                        calculated_path.as_os_str().to_str().unwrap()
                    )))
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&format!(
                "Cannot open conntrolComponentPubllicKeysPayload.{}.json: {}",
                i, e
            ))),
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
}
