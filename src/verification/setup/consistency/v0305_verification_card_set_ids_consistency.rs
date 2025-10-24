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

fn verrify_card_set_ids_context_vcs<V: ContextVCSDirectoryTrait>(
    vcs_dir: &V,
) -> VerificationResult {
    let mut res = VerificationResult::new();
    let vcs_id = vcs_dir.name();
    match vcs_dir.setup_component_tally_data_payload() {
        Ok(p) => {
            if p.verification_card_set_id != vcs_id {
                res.push(
                VerificationEvent::new_failure(
                    &format!(
                        "verification card set in file setup_component_tally_data_payload doesn't match with expected {}",
                        vcs_id
                    )
                )
            );
            }
        }
        Err(e) => res.push(
            VerificationEvent::new_error_from_error(&e)
                .add_context("Cannot read payload for setup_component_tally_data_payload"),
        ),
    }
    res
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();

    for vcs_dir in context_dir.vcs_directories().iter() {
        result.append_with_context(
            &verrify_card_set_ids_context_vcs(vcs_dir),
            format!("context vcs directory {}", vcs_dir.name()),
        );
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
                    d.verification_card_set_id = "modified-vcs_id".to_string();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors(), "Failed at vcs {}", vcs.name());
            assert!(result.has_failures(), "Failed at VCS {}", vcs.name());
        }
    }
}
