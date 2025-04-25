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
    data_structures::{
        ControlComponentBallotBoxPayload, ControlComponentShufflePayload,
        TallyComponentShufflePayload,
    },
    file_structure::{
        tally_directory::BBDirectoryTrait, ContextDirectoryTrait, TallyDirectoryTrait,
        VerificationDirectoryTrait,
    },
};
use rust_ev_system_library::preliminaries::PTableTrait;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::elgamal::Ciphertext;

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
                VerificationEvent::new_error(&e)
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

    for (i, cc_bb_payload_res) in bb_dir.control_component_ballot_box_payload_iter() {
        match cc_bb_payload_res {
            Ok(cc_bb_payload) => res.append_with_context(
                &verify_cc_bb_payload(&cc_bb_payload, delta),
                format!(
                    "{}/control_component_ballot_box_payload_.{}",
                    bb_dir.name(),
                    i
                ),
            ),
            Err(e) => res.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_ballot_box_payload_.{} has wrong format",
                bb_dir.name(),
                i
            ))),
        }
    }

    for (i, cc_shuffle_payload_res) in bb_dir.control_component_shuffle_payload_iter() {
        match cc_shuffle_payload_res {
            Ok(cc_shuffle_payload) => res.append_with_context(
                &verify_cc_shuffle_payload(&cc_shuffle_payload, delta),
                format!(
                    "{}/control_component_ballot_box_payload_.{}",
                    bb_dir.name(),
                    i
                ),
            ),
            Err(e) => res.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_ballot_box_payload_.{} has wrong format",
                bb_dir.name(),
                i
            ))),
        }
    }

    match bb_dir.tally_component_shuffle_payload() {
        Ok(tally_shuffle_payload) => res.append_with_context(
            &verify_tally_shuffle_payload(&tally_shuffle_payload, delta),
            format!("{}/tally_component_shuffle_payload", bb_dir.name(),),
        ),
        Err(e) => res.push(VerificationEvent::new_error(&e).add_context(format!(
            "{}/tally_component_shuffle_payload has wrong format",
            bb_dir.name(),
        ))),
    }
    res
}

fn verify_cc_bb_payload(
    payload: &ControlComponentBallotBoxPayload,
    delta: usize,
) -> VerificationResult {
    verify_vec_ciphertexts(
        payload
            .confirmed_encrypted_votes
            .iter()
            .map(|v| &v.encrypted_vote)
            .collect::<Vec<_>>()
            .as_slice(),
        delta,
    )
}

fn verify_cc_shuffle_payload(
    payload: &ControlComponentShufflePayload,
    delta: usize,
) -> VerificationResult {
    verify_vec_ciphertexts(
        payload
            .verifiable_shuffle
            .shuffled_ciphertexts
            .iter()
            .collect::<Vec<_>>()
            .as_slice(),
        delta,
    )
}

fn verify_tally_shuffle_payload(
    payload: &TallyComponentShufflePayload,
    delta: usize,
) -> VerificationResult {
    verify_vec_ciphertexts(
        payload
            .verifiable_shuffle
            .shuffled_ciphertexts
            .iter()
            .collect::<Vec<_>>()
            .as_slice(),
        delta,
    )
}

fn verify_vec_ciphertexts(data: &[&Ciphertext], delta: usize) -> VerificationResult {
    VerificationResult::from(data
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            if v.phis.len() == delta {
                None
            } else {
                Some(VerificationEvent::new_failure(&format!("At pos {}: number of ciphertext elements doesn't equal the number allowed write-ins plus one", i)))
            }
        }).collect::<Vec<_>>().as_slice())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_tally_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
