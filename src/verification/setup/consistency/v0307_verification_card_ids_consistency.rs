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
    data_structures::context::{
        election_event_context_payload::ElectionEventContext,
        setup_component_tally_data_payload::SetupComponentTallyDataPayload,
    },
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use std::collections::HashSet;

fn verrify_card_ids_context_vcs(
    ee_context_payload: &ElectionEventContext,
    setup_component_public_keys_payload: &SetupComponentTallyDataPayload,
    unique_set: &mut HashSet<String>,
) -> VerificationResult {
    let mut res = VerificationResult::new();
    match ee_context_payload
        .verification_card_set_contexts
        .iter()
        .find(|vcs| {
            vcs.verification_card_set_id
                == setup_component_public_keys_payload.verification_card_set_id
        }) {
        Some(c) => {
            if c.number_of_eligible_voters
                != setup_component_public_keys_payload
                    .verification_card_ids
                    .len()
            {
                res.push(VerificationEvent::new_failure(&format!(
                "The vcnumber of voting card ids {} is not the same that the number of elligible voters {}",
                setup_component_public_keys_payload.verification_card_ids.len(), c.number_of_eligible_voters)));
            }
        }
        None => {
            res.push(VerificationEvent::new_error(&format!(
                "voting card set context with id {} not found in election event context",
                &setup_component_public_keys_payload.verification_card_set_id
            )));
        }
    };
    for vc_id in setup_component_public_keys_payload
        .verification_card_ids
        .iter()
    {
        if !unique_set.insert(vc_id.clone()) {
            res.push(VerificationEvent::new_failure(&format!(
                "The vc_id {} is not unique",
                vc_id
            )));
        }
    }
    res
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();

    let ee_context_payload = match context_dir.election_event_context_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot extract election_event_context_payload"),
            );
            return;
        }
    };

    let mut uniq = HashSet::new();

    for vcs_dir in context_dir.vcs_directories().iter() {
        match vcs_dir.setup_component_tally_data_payload() {
            Ok(p) => {
                result.append_with_context(
                    &verrify_card_ids_context_vcs(
                        &ee_context_payload.election_event_context,
                        p.as_ref(),
                        &mut uniq,
                    ),
                    format!("context vcs directory {}", vcs_dir.name()),
                );
            }
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e).add_context(format!(
                        "Cannot read payload for {}/setup_component_tally_data_payload",
                        vcs_dir.name()
                    )),
                );
            }
        };
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
