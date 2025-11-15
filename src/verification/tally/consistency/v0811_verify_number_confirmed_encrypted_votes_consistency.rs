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
        TallyDirectoryTrait, VerificationDirectoryTrait, tally_directory::BBDirectoryTrait,
    },
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();

    for bb_dir in tally_dir.bb_directories().iter() {
        result.append_with_context(
            &verify_for_bb_directory(bb_dir),
            format!("Ballot box directory {}", bb_dir.name()),
        );
    }
}

fn verify_for_bb_directory<B: BBDirectoryTrait>(bb_dir: &B) -> VerificationResult {
    let mut result = VerificationResult::new();

    let bb_name = bb_dir.name();

    let nb_votes = match bb_dir.tally_component_votes_payload() {
        Ok(p) => p.decrypted_votes.len(),
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{}/tally_component_votes_payload cannot be read",
                    bb_name
                )),
            );
            return result;
        }
    };

    let nb_mixed_votes = match bb_dir.tally_component_shuffle_payload() {
        Ok(p) => p.verifiable_shuffle.shuffled_ciphertexts.len(),
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{}/tally_component_shuffle_payload cannot be read",
                    bb_name
                )),
            );
            return result;
        }
    };

    for (i, cc_bb_payload_res) in bb_dir.control_component_ballot_box_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => {
                if p.confirmed_encrypted_votes.len() != nb_votes {
                    result.push(VerificationEvent::new_failure(&format!(
                    "The number of vote {} in {}/control_component_shuffle_payload_{} is not the same than the number of votes {} in tally_component_votes_payload",
                    p.confirmed_encrypted_votes.len(), bb_name, i, nb_votes
                )));
                }
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "{}/control_component_ballot_box_payload_{} cannot be read",
                    bb_name, i
                ),
            )),
        }
    }

    for (i, cc_bb_payload_res) in bb_dir.control_component_shuffle_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => {
                if nb_mixed_votes != p.verifiable_decryptions.ciphertexts.len() {
                    result.push(VerificationEvent::new_failure(&format!(
                    "The number of mixed vote {} in {}/control_component_shuffle_payload_{} is not the same than the number of mixed votes {} in tally_component_shuffle_payload",
                    p.verifiable_decryptions.ciphertexts.len(), bb_name, i, nb_mixed_votes
                )));
                }
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "{}/control_component_shuffle_payload_{} cannot be read",
                    bb_name, i
                ),
            )),
        }
    }

    match nb_votes {
        n if n < 2 => {
            if nb_mixed_votes != n + 2 {
                result.push(VerificationEvent::new_failure(&format!(
                "The number of mixed votes {} in {}/tally_component_shuffle_payload is not equal to the number of votes {} in tally_component_votes_payload plus two",
                nb_mixed_votes, bb_name, n
            )));
            }
        }
        n => {
            if nb_mixed_votes != n {
                result.push(VerificationEvent::new_failure(&format!(
                "The number of mixed votes {} in {}/tally_component_shuffle_payload is not equal to the number of votes {} in tally_component_votes_payload",
                nb_mixed_votes, bb_name, nb_votes
            )));
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        config::test::{
            CONFIG_TEST, get_test_verifier_mock_tally_dir,
            get_test_verifier_tally_dir as get_verifier_dir,
        },
        consts::NUMBER_CONTROL_COMPONENTS,
    };

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
    fn add_vote_tally() {
        let dir = get_verifier_dir();
        let nb = dir.unwrap_tally().bb_directories().len();
        for i in 0..nb {
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                .mock_tally_component_votes_payload(|d| d.decrypted_votes.push(vec![1usize; 10]));
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(
                !result.has_errors(),
                "Failed for decrypted votes for bb id {i}"
            );
            assert!(
                result.has_failures(),
                "Failed for decrypted votes for bb id {i}"
            );
        }
    }

    #[test]
    fn remove_vote_tally() {
        let dir = get_verifier_dir();
        let nb = dir.unwrap_tally().bb_directories().len();
        for i in 0..nb {
            let bb = &dir.unwrap_tally().bb_directories()[i];
            if bb
                .tally_component_votes_payload()
                .unwrap()
                .decrypted_votes
                .is_empty()
            {
                continue;
            }
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i].mock_tally_component_votes_payload(
                |d| {
                    d.decrypted_votes.pop();
                },
            );
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(
                !result.has_errors(),
                "Failed for decrypted votes for bb {}",
                bb.name()
            );
            assert!(
                result.has_failures(),
                "Failed for decrypted votes for bb {}",
                bb.name()
            );
        }
    }

    #[test]
    fn remove_vote_cc() {
        let dir = get_verifier_dir();
        let nb = dir.unwrap_tally().bb_directories().len();
        for i in 0..nb {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                let bb = &dir.unwrap_tally().bb_directories()[i];
                if bb
                    .control_component_ballot_box_payload_iter()
                    .nth(j - 1)
                    .unwrap()
                    .1
                    .as_ref()
                    .unwrap()
                    .confirmed_encrypted_votes
                    .is_empty()
                {
                    continue;
                }
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_ballot_box_payload(j, |d| {
                        d.confirmed_encrypted_votes.pop();
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(
                    !result.has_errors(),
                    "Failed for decrypted votes for bb {} and cc {j}",
                    bb.name()
                );
                assert!(
                    result.has_failures(),
                    "Failed for decrypted votes for bb {} and cc {j}",
                    bb.name()
                );
            }
        }
    }

    #[test]
    fn remove_mixed_vote_tally() {
        let dir = get_verifier_dir();
        let nb = dir.unwrap_tally().bb_directories().len();
        for i in 0..nb {
            let bb = &dir.unwrap_tally().bb_directories()[i];
            if bb
                .tally_component_shuffle_payload()
                .unwrap()
                .verifiable_shuffle
                .shuffled_ciphertexts
                .is_empty()
            {
                continue;
            }
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                .mock_tally_component_shuffle_payload(|d| {
                    d.verifiable_shuffle.shuffled_ciphertexts.pop();
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(
                !result.has_errors(),
                "Failed for decrypted votes for bb {}",
                bb.name()
            );
            assert!(
                result.has_failures(),
                "Failed for decrypted votes for bb {}",
                bb.name()
            );
        }
    }

    #[test]
    fn remove_mixed_vote_cc() {
        let dir = get_verifier_dir();
        let nb = dir.unwrap_tally().bb_directories().len();
        for i in 0..nb {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                let bb = &dir.unwrap_tally().bb_directories()[i];
                if bb
                    .control_component_shuffle_payload_iter()
                    .nth(j - 1)
                    .unwrap()
                    .1
                    .as_ref()
                    .unwrap()
                    .verifiable_decryptions
                    .ciphertexts
                    .is_empty()
                {
                    continue;
                }
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_shuffle_payload(j, |d| {
                        d.verifiable_decryptions.ciphertexts.pop();
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(
                    !result.has_errors(),
                    "Failed for decrypted votes for bb {} and cc {j}",
                    bb.name()
                );
                assert!(
                    result.has_failures(),
                    "Failed for decrypted votes for bb {} and cc {j}",
                    bb.name()
                );
            }
        }
    }
}
