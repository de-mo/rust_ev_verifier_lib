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

mod verify_ech0222;
mod verify_process_plaintexts;
mod verify_tally_control_component_ballot_box;

use crate::{
    data_structures::ElectionEventContextPayload,
    file_structure::{
        tally_directory::BBDirectoryTrait, ContextDirectoryTrait, TallyDirectoryTrait,
        VerificationDirectoryTrait,
    },
    verification::{VerificationEvent, VerificationResult},
    VerifierConfig,
};
use rayon::prelude::*;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    mix_net::ShuffleArgument, Integer,
};
use verify_tally_control_component_ballot_box::{
    verify_tally_control_component_ballot_box, ContextAlgorithm42, InputsAlgorithm42,
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let tally_dir = dir.unwrap_tally();

    let ee_context_payload = match context_dir.election_event_context_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };

    let setup_pk_payload = match context_dir.setup_component_public_keys_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("setup_component_public_keys_payload cannot be read"),
            );
            return;
        }
    };
    let eb_pk = &setup_pk_payload
        .setup_component_public_keys
        .electoral_board_public_key
        .as_slice();

    result.append(
        &mut tally_dir
            .bb_directories()
            .par_iter()
            .map(|dir| {
                (
                    dir.name(),
                    verify_for_ballotbox(&ee_context_payload, eb_pk, dir),
                )
            })
            .collect::<Vec<_>>()
            .iter()
            .fold(VerificationResult::new(), |acc, (name, result)| {
                let mut res = acc.clone();
                res.append_with_context(result, format!("Ballot box {name}"));
                res
            }),
    );

    {
        let ee_configuration = match context_dir.election_event_configuration() {
            Ok(p) => p,
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e)
                        .add_context("election_event_configuration cannot be read"),
                );
                return;
            }
        };

        let ee_configuration_data = match ee_configuration.get_data() {
            Ok(d) => d,
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e)
                        .add_context("election_event_configuration data cannot be parsed"),
                );
                return;
            }
        };

        let ech_0222 = match tally_dir.ech_0222() {
            Ok(p) => p,
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e)
                        .add_context("ech_0222_payload cannot be read"),
                );
                return;
            }
        };

        let ech_0222_data = match ech_0222.get_data() {
            Ok(d) => d,
            Err(e) => {
                result.push(
                    VerificationEvent::new_error_from_error(&e)
                        .add_context("ech_0222_payload data cannot be parsed"),
                );
                return;
            }
        };

        result.append(&mut verify_ech0222::verify_ech0222(
            &ee_context_payload.election_event_context,
            &ee_configuration_data,
            ech_0222_data.as_ref(),
            tally_dir.bb_directories(),
        ));
    };
}

fn verify_for_ballotbox<B: BBDirectoryTrait>(
    ee_context_payload: &ElectionEventContextPayload,
    eb_pk: &[Integer],
    tally_dir: &B,
) -> VerificationResult {
    let bb_id = tally_dir.name();
    let vcs = match ee_context_payload
        .election_event_context
        .find_verification_card_set_context_with_bb_id(&bb_id)
    {
        Some(vcs) => vcs,
        None => {
            return VerificationResult::from(&VerificationEvent::new_error(&format!(
                "No verification card set found for ballot box {bb_id}"
            )))
        }
    };

    let context_42 = ContextAlgorithm42 {
        eg: &ee_context_payload.encryption_group,
        ee_id: &ee_context_payload.election_event_context.election_event_id,
        bb_id: &bb_id,
        _upper_n_upper_e: vcs.number_of_eligible_voters,
        p_table: &vcs.primes_mapping_table.p_table,
        eb_pk,
    };

    let cc_shuffle_payload_4 = match tally_dir
        .control_component_shuffle_payload_iter()
        .find(|(i, _)| *i == 4)
    {
        Some((_, p)) => match p {
            Ok(p) => p,
            Err(e) => {
                return VerificationResult::from(
                    &VerificationEvent::new_error_from_error(&e).add_context(format!(
                        "{bb_id}/control_component_shuffle_payload_4 cannot be read"
                    )),
                );
            }
        },
        None => {
            return VerificationResult::from(&VerificationEvent::new_error(&format!(
                "{bb_id}/tally_component_shuffle_payload_4 not found",
            )));
        }
    };

    let tally_shuffle_payload = match tally_dir.tally_component_shuffle_payload() {
        Ok(p) => p,
        Err(e) => {
            return VerificationResult::from(
                &VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{bb_id}/tally_component_shuffle_payload cannot be read"
                )),
            );
        }
    };

    let tally_votes_payload = match tally_dir.tally_component_votes_payload() {
        Ok(p) => p,
        Err(e) => {
            return VerificationResult::from(
                &VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{bb_id}/tally_component_votes_payload cannot be read"
                )),
            );
        }
    };
    let pi_mix_5 =
        match ShuffleArgument::try_from(&tally_shuffle_payload.verifiable_shuffle.shuffle_argument)
        {
            Ok(a) => a,
            Err(e) => return VerificationResult::from(&VerificationEvent::new_error_from_error(&e).add_context(
                format!("Error converting shuffle argument for {bb_id}/tally_component_shuffle_payload cannot be read", ),
            )),
        };

    let decrypted_votes = tally_shuffle_payload
        .verifiable_plaintext_decryption
        .decrypted_votes
        .iter()
        .map(|v| v.message.as_slice())
        .collect::<Vec<_>>();

    let input_42 = InputsAlgorithm42 {
        c_dec_4: &cc_shuffle_payload_4.verifiable_decryptions.ciphertexts,
        c_mix_5: &tally_shuffle_payload
            .verifiable_shuffle
            .shuffled_ciphertexts,
        pi_mix_5: &pi_mix_5,
        ms: &decrypted_votes,
        pi_dec_5: &tally_shuffle_payload
            .verifiable_plaintext_decryption
            .decryption_proofs,
        upper_l_votes: &tally_votes_payload.decrypted_votes,
        upper_l_decoded_votes: &tally_votes_payload.decoded_votes,
        upper_l_write_ins: &tally_votes_payload.decoded_write_ins,
    };

    verify_tally_control_component_ballot_box(&context_42, &input_42)
        .clone_add_context("VerifyTallyControlComponentBallotBox")
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
}
