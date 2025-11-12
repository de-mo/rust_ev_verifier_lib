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

mod consistent_xml;

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    data_structures::context::election_event_context_payload::ElectionEventContext,
    file_structure::{VerificationDirectoryTrait, context_directory::ContextDirectoryTrait},
};
use consistent_xml::verification_2_3_same_than_xml;
use rust_ev_system_library::preliminaries::PTableElement;
use std::collections::HashMap;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let ee_c_paylod = match context_dir.election_event_context_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot extract election_event_context_payload"),
            );
            return;
        }
    };

    let ee_configuration_data = match context_dir.election_event_configuration() {
        Ok(o) => match o.get_data() {
            Ok(o) => o,
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e)
                        .add_context("Cannot extract election_event_configuration"),
                );
                return;
            }
        },
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot read election_event_configuration"),
            );
            return;
        }
    };

    // Verification 1
    result.append_with_context(
        &verify_1_same_actual_voting_options(&ee_c_paylod.election_event_context),
        "Verification 1 (same actual voting option v_i maps to the same element of the pTable)",
    );

    // Verifications 2 and 3
    result.append_with_context(
        &verification_2_3_same_than_xml(
            &ee_c_paylod.election_event_context,
            ee_configuration_data.as_ref(),
        ),
        "Verification 2 and 3 (consistent to xml)",
    );
}

/// Verification 1 according to the specification of Swiss Post
/// The same actual voting option v_i maps to the same element of the pTable
fn verify_1_same_actual_voting_options(ee_context: &ElectionEventContext) -> VerificationResult {
    let mut result = VerificationResult::new();
    let mut actual_hashmaps: HashMap<String, &PTableElement> = HashMap::new();
    for vcs_context in ee_context.verification_card_set_contexts.iter() {
        let vcs_id = vcs_context.verification_card_set_id.as_str();
        for p_table_element in vcs_context.primes_mapping_table.p_table.iter() {
            match actual_hashmaps.get(&p_table_element.actual_voting_option) {
                Some(found_p_table_element) => {
                    if found_p_table_element != &p_table_element {
                        result.push(VerificationEvent::new_failure(&format!(
                            "The actual voting option {} for vcs_id{} is not the same as the previous one\n Found: {}\n Expected: {}",
                            p_table_element.actual_voting_option, vcs_id, found_p_table_element, p_table_element
                        )));
                    }
                }
                None => {
                    let _ = actual_hashmaps.insert(
                        p_table_element.actual_voting_option.clone(),
                        p_table_element,
                    );
                }
            };
        }
    }
    result
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
        if !result.is_ok() {
            for r in result.errors_to_string() {
                println!("{r:?}")
            }
            for r in result.failures_to_string() {
                println!("{r:?}")
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn change_p_table_element() {
        let dir = get_verifier_dir();
        for (i, vcs) in dir
            .context()
            .election_event_context_payload()
            .unwrap()
            .election_event_context
            .verification_card_set_contexts
            .iter()
            .enumerate()
        {
            for (j, _p_table_element) in vcs.primes_mapping_table.p_table.iter().enumerate() {
                {
                    // encoded_voting_option change
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_setup_dir();
                    mock_dir
                        .context_mut()
                        .mock_election_event_context_payload(|d| {
                            d.election_event_context.verification_card_set_contexts[i]
                                .primes_mapping_table
                                .p_table[j]
                                .encoded_voting_option = 1usize
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(
                        !result.has_errors(),
                        "Failed at vcs {} in position {j} of pTable for encoded_voting_option change",
                        &vcs.verification_card_set_id
                    );
                    assert!(
                        result.has_failures(),
                        "Failed at VCS {} in position {j} of pTable for encoded_voting_option change",
                        &vcs.verification_card_set_id
                    );
                }
                {
                    // actual_voting_option change
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_setup_dir();
                    mock_dir
                        .context_mut()
                        .mock_election_event_context_payload(|d| {
                            d.election_event_context.verification_card_set_contexts[i]
                                .primes_mapping_table
                                .p_table[j]
                                .actual_voting_option = "Changed".to_string()
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(
                        !result.has_errors(),
                        "Failed at vcs {} in position {j} of pTable for actual_voting_option change",
                        &vcs.verification_card_set_id
                    );
                    assert!(
                        result.has_failures(),
                        "Failed at VCS {} in position {j} of pTable for actual_voting_option change",
                        &vcs.verification_card_set_id
                    );
                }
                {
                    // semantic_information change
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_setup_dir();
                    mock_dir
                        .context_mut()
                        .mock_election_event_context_payload(|d| {
                            d.election_event_context.verification_card_set_contexts[i]
                                .primes_mapping_table
                                .p_table[j]
                                .semantic_information = "Changed".to_string()
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(
                        !result.has_errors(),
                        "Failed at vcs {} in position {j} of pTable for semantic_information change",
                        &vcs.verification_card_set_id
                    );
                    assert!(
                        result.has_failures(),
                        "Failed at VCS {} in position {j} of pTable for semantic_information change",
                        &vcs.verification_card_set_id
                    );
                }
                {
                    // correctness_information change
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_setup_dir();
                    mock_dir
                        .context_mut()
                        .mock_election_event_context_payload(|d| {
                            d.election_event_context.verification_card_set_contexts[i]
                                .primes_mapping_table
                                .p_table[j]
                                .correctness_information = "Changed".to_string()
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(
                        !result.has_errors(),
                        "Failed at vcs {} in position {j} of pTable for correctness_information change",
                        &vcs.verification_card_set_id
                    );
                    assert!(
                        result.has_failures(),
                        "Failed at VCS {} in position {j} of pTable for correctness_information change",
                        &vcs.verification_card_set_id
                    );
                }
            }
        }
    }

    #[test]
    fn remove_p_table_element() {
        let dir = get_verifier_dir();
        for (i, vcs) in dir
            .context()
            .election_event_context_payload()
            .unwrap()
            .election_event_context
            .verification_card_set_contexts
            .iter()
            .enumerate()
        {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_election_event_context_payload(|d| {
                    d.election_event_context.verification_card_set_contexts[i]
                        .primes_mapping_table
                        .p_table
                        .pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(
                !result.has_errors(),
                "Failed at vcs {}",
                &vcs.verification_card_set_id
            );
            assert!(
                result.has_failures(),
                "Failed at VCS {}",
                &vcs.verification_card_set_id
            );
        }
    }

    #[test]
    fn add_p_table_element() {
        let dir = get_verifier_dir();
        for (i, vcs) in dir
            .context()
            .election_event_context_payload()
            .unwrap()
            .election_event_context
            .verification_card_set_contexts
            .iter()
            .enumerate()
        {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_setup_dir();
            mock_dir
                .context_mut()
                .mock_election_event_context_payload(|d| {
                    d.election_event_context.verification_card_set_contexts[i]
                        .primes_mapping_table
                        .p_table
                        .push(PTableElement {
                            actual_voting_option: "New".to_string(),
                            encoded_voting_option: 1usize,
                            semantic_information: "New".to_string(),
                            correctness_information: "New".to_string(),
                        })
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(
                !result.has_errors(),
                "Failed at vcs {}",
                &vcs.verification_card_set_id
            );
            assert!(
                result.has_failures(),
                "Failed at VCS {}",
                &vcs.verification_card_set_id
            );
        }
    }
}
