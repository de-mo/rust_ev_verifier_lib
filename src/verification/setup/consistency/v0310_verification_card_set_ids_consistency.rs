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
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};

fn verrify_card_set_ids_setup_vcs<V: SetupVCSDirectoryTrait>(vcs_dir: &V) -> VerificationResult {
    let mut res = VerificationResult::new();
    let vcs_id = vcs_dir.name();
    for (chunk, payload_res) in vcs_dir.setup_component_verification_data_payload_iter() {
        if let Err(e) = payload_res {
            res.push(VerificationEvent::new_error(&e).add_context(format!(
                "Cannot read payload for setup_component_verification_data.{}",
                chunk
            )));
            break;
        }
        if payload_res.unwrap().verification_card_set_id != vcs_id {
            res.push(
                VerificationEvent::new_failure(
                    &format!(
                        "verification card set in file setup_component_verification_data.{} doesn't match with expected {}",
                        chunk,
                        vcs_id
                    )
                )
            );
        }
    }

    for (chunk, payload_res) in vcs_dir.control_component_code_shares_payload_iter() {
        if let Err(e) = payload_res {
            res.push(VerificationEvent::new_error(&e).add_context(format!(
                "Cannot read payload for setup_component_verification_data.{}",
                chunk
            )));
            break;
        }
        for payload in payload_res.unwrap().0.iter() {
            if payload.verification_card_set_id != vcs_id {
                res.push(
                    VerificationEvent::new_failure(
                        &format!(
                            "verification card set for node {} in file setup_component_verification_data.{} doesn't match with expected {}",
                            payload.node_id,
                            chunk,
                            vcs_id
                        )
                    )
                );
            }
        }
    }
    res
}

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
            VerificationEvent::new_error(&e)
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
    let setup_dir = dir.unwrap_setup();

    for vcs_dir in context_dir.vcs_directories().iter() {
        result.append_with_context(
            &verrify_card_set_ids_context_vcs(vcs_dir),
            format!("context vcs directory {}", vcs_dir.name()),
        );
    }

    for vcs_dir in setup_dir.vcs_directories().iter() {
        result.append_with_context(
            &verrify_card_set_ids_setup_vcs(vcs_dir),
            format!("setup vcs directory {}", vcs_dir.name()),
        );
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
