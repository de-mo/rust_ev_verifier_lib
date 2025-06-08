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

fn test_election_event_id(ee_id: &String, expected: &String) -> VerificationResult {
    let mut result = VerificationResult::new();
    if ee_id != expected {
        result.push(VerificationEvent::new_failure(&format!(
            "Election Event ID {} not equal to {}",
            ee_id, expected
        )));
    }
    result
}

fn test_ee_id_for_context_vcs_dir<V: ContextVCSDirectoryTrait>(
    dir: &V,
    expected: &String,
    result: &mut VerificationResult,
) {
    match dir.setup_component_tally_data_payload() {
        Ok(p) => result.append_with_context(
            &test_election_event_id(&p.election_event_id, expected),
            format!("{}/setup_component_tally_data_payload", dir.name()),
        ),
        Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
            "{}/setup_component_tally_data_payload has wrong format",
            dir.name()
        ))),
    }
}

fn test_ee_id_for_setup_vcs_dir<V: SetupVCSDirectoryTrait>(
    dir: &V,
    expected: &String,
    result: &mut VerificationResult,
) {
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        match f {
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/control_component_code_shares_payload_.{} has wrong format",
                dir.name(),
                i
            ))),
            Ok(cc) => {
                for p in cc.0.iter() {
                    result.append_with_context(
                        &test_election_event_id(&p.election_event_id, expected),
                        format!(
                            "{}/control_component_code_shares_payload.{}_chunk{}",
                            dir.name(),
                            i,
                            p.chunk_id
                        ),
                    )
                }
            }
        }
    }
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        match f {
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/setup_component_verification_data_payload.{} has wrong format",
                dir.name(),
                i
            ))),
            Ok(s) => result.append_with_context(
                &test_election_event_id(&s.election_event_id, expected),
                format!(
                    "{}/setup_component_verification_data_payload.{}",
                    i,
                    dir.name()
                ),
            ),
        }
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_dir = dir.unwrap_setup();
    let context = match context_dir.election_event_context_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    let ee_id = &context.as_ref().election_event_context.election_event_id;
    match context_dir.setup_component_public_keys_payload() {
        Ok(p) => result.append_with_context(
            &test_election_event_id(&p.election_event_id, ee_id),
            "setup_component_public_keys_payload",
        ),
        Err(e) => result.push(
            VerificationEvent::new_error_from_error(&e)
                .add_context("election_event_context_payload has wrong format"),
        ),
    }
    for (i, f) in context_dir.control_component_public_keys_payload_iter() {
        match f {
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                "control_component_public_keys_payload.{} has wrong format",
                i
            ))),
            Ok(cc) => result.append_with_context(
                &test_election_event_id(&cc.election_event_id, ee_id),
                format!("control_component_public_keys_payload.{}", i),
            ),
        }
    }
    for vcs in context_dir.vcs_directories().iter() {
        test_ee_id_for_context_vcs_dir(vcs, ee_id, result);
    }
    for vcs in setup_dir.vcs_directories().iter() {
        test_ee_id_for_setup_vcs_dir(vcs, ee_id, result);
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
