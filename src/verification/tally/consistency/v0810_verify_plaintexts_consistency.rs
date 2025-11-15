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
        ContextDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait,
        tally_directory::BBDirectoryTrait,
    },
};
use rust_ev_system_library::preliminaries::PTableTrait;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let tally_dir = dir.unwrap_tally();

    let payload = match context_dir.election_event_context_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot extract election_event_context_payload"),
            );
            return;
        }
    };
    let vcs_contexts = &payload
        .election_event_context
        .verification_card_set_contexts;

    let mut res = VerificationResult::join(
        tally_dir
            .bb_directories()
            .iter()
            .map(
                |dir| match vcs_contexts.iter().find(|c| c.ballot_box_id == dir.name()) {
                    Some(c) => {
                        verify_pro_ballot_box(dir, c.primes_mapping_table.p_table.get_delta())
                            .clone_add_context(format!("ballot box {}", dir.name()))
                    }
                    None => VerificationResult::from(&VerificationEvent::new_error(&format!(
                        "context for ballot box id {} not found",
                        dir.name()
                    ))),
                },
            )
            .collect::<Vec<_>>()
            .as_slice(),
    );
    result.append(&mut res);
}

fn verify_pro_ballot_box<B: BBDirectoryTrait>(bb_dir: &B, delta: usize) -> VerificationResult {
    let mut res = VerificationResult::new();

    let tally_shuffle_payload = match bb_dir.tally_component_shuffle_payload() {
        Ok(tally_shuffle_payload) => tally_shuffle_payload,
        Err(e) => {
            return VerificationResult::from(
                &VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{}/tally_component_shuffle_payload has wrong format",
                    bb_dir.name(),
                )),
            );
        }
    };

    res.append(&mut VerificationResult::from(
        tally_shuffle_payload
            .verifiable_plaintext_decryption
            .decrypted_votes
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if v.message.len() == delta {
                    None
                } else {
                    Some(VerificationEvent::new_failure(&format!(
                    "size of message of decrypted vote at pos {} is not the same as delta + 1 = {}",
                    i,
                    delta + 1
                )))
                }
            })
            .collect::<Vec<_>>()
            .as_slice(),
    ));

    res.append(&mut VerificationResult::from(
        tally_shuffle_payload
            .verifiable_plaintext_decryption
            .decryption_proofs
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.z.len() == delta {
                    None
                } else {
                    Some(VerificationEvent::new_failure(&format!(
                        "size of message of proofs at pos {} is not the same as delta + 1 = {}",
                        i,
                        delta + 1
                    )))
                }
            })
            .collect::<Vec<_>>()
            .as_slice(),
    ));
    res
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        CONFIG_TEST, get_test_verifier_mock_tally_dir,
        get_test_verifier_tally_dir as get_verifier_dir,
    };
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for r in result.errors_to_string() {
                println!("{:?}", r)
            }
            for r in result.failures_to_string() {
                println!("{:?}", r)
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn remove_plaintext_tally_shuffle() {
        let dir = get_verifier_dir();
        let nb = dir.unwrap_tally().bb_directories().len();
        for i in 0..nb {
            for c_i in 0..dir.unwrap_tally().bb_directories()[i]
                .tally_component_shuffle_payload()
                .unwrap()
                .verifiable_plaintext_decryption
                .decrypted_votes
                .len()
            {
                {
                    // Decrypted votes
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_tally_dir();
                    mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                        .mock_tally_component_shuffle_payload(|d| {
                            d.verifiable_plaintext_decryption.decrypted_votes[c_i]
                                .message
                                .pop();
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(
                        !result.has_errors(),
                        "Failed for decrypted votes at position {c_i}"
                    );
                    assert!(
                        result.has_failures(),
                        "Failed for decrypted votes at position {c_i}"
                    );
                }
                {
                    // Decryption proofs
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_tally_dir();
                    mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                        .mock_tally_component_shuffle_payload(|d| {
                            d.verifiable_plaintext_decryption.decryption_proofs[c_i]
                                .z
                                .pop();
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(
                        !result.has_errors(),
                        "Failed for decrypted votes at position {c_i}"
                    );
                    assert!(
                        result.has_failures(),
                        "Failed for decrypted votes at position {c_i}"
                    );
                }
            }
        }
    }

    #[test]
    fn add_plaintext_tally_shuffle() {
        let dir = get_verifier_dir();
        let nb = dir.unwrap_tally().bb_directories().len();
        for i in 0..nb {
            for c_i in 0..dir.unwrap_tally().bb_directories()[i]
                .tally_component_shuffle_payload()
                .unwrap()
                .verifiable_plaintext_decryption
                .decrypted_votes
                .len()
            {
                {
                    // Decrypted votes
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_tally_dir();
                    mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                        .mock_tally_component_shuffle_payload(|d| {
                            d.verifiable_plaintext_decryption.decrypted_votes[c_i]
                                .message
                                .push(Integer::from(123usize));
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(!result.has_errors());
                    assert!(result.has_failures());
                }
                {
                    // Decrypted proofs
                    let mut result = VerificationResult::new();
                    let mut mock_dir = get_test_verifier_mock_tally_dir();
                    mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                        .mock_tally_component_shuffle_payload(|d| {
                            d.verifiable_plaintext_decryption.decryption_proofs[c_i]
                                .z
                                .push(Integer::from(123usize));
                        });
                    fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                    assert!(!result.has_errors());
                    assert!(result.has_failures());
                }
            }
        }
    }
}
