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

use super::verify_process_plaintexts::{
    verify_process_plaintexts, ContextAlgorithm43, InputsAlgorithm43,
};
use crate::{
    data_structures::common_types::DecryptionProof,
    verification::{VerificationEvent, VerificationResult},
};
use rust_ev_system_library::preliminaries::{PTable, PTableTrait};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::{Ciphertext, EncryptionParameters},
    mix_net::{verify_shuffle, MixNetResultTrait, ShuffleArgument},
    zero_knowledge_proofs::verify_decryption,
    Integer,
};

/// Context data for algorithm 4.2 according to the specifications
pub struct ContextAlgorithm42<'a> {
    pub eg: &'a EncryptionParameters,
    pub ee_id: &'a str,
    pub bb_id: &'a str,
    pub _upper_n_upper_e: usize,
    pub p_table: &'a PTable,
    pub eb_pk: &'a [Integer],
}

pub struct InputsAlgorithm42<'a> {
    pub c_dec_4: &'a [Ciphertext],
    pub c_mix_5: &'a [Ciphertext],
    pub pi_mix_5: &'a ShuffleArgument<'a>,
    pub ms: &'a [&'a [Integer]],
    pub pi_dec_5: &'a [DecryptionProof],
    pub upper_l_votes: &'a [Vec<usize>],
    pub upper_l_decoded_votes: &'a [Vec<String>],
    pub upper_l_write_ins: &'a [Vec<String>],
}

pub fn verify_tally_control_component_ballot_box<'a>(
    context: &ContextAlgorithm42<'a>,
    input: &InputsAlgorithm42<'a>,
) -> VerificationResult {
    let mut res = VerificationResult::new();
    let i_aux = [context.ee_id, context.bb_id, "MixDecOffline"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let eb_pk_cut = context
        .eb_pk
        .iter()
        .take(context.p_table.get_delta())
        .cloned()
        .collect::<Vec<_>>();

    match verify_shuffle(
        context.eg,
        input.c_dec_4,
        input.c_mix_5,
        input.pi_mix_5,
        eb_pk_cut.as_slice(),
    ) {
        Ok(r) => {
            if !r.is_ok() {
                res.push_with_context(
                    VerificationEvent::new_failure(&r.to_string()),
                    format!("VerifyShuffle for bb {}", context.bb_id),
                );
            }
        }
        Err(e) => res.push_with_context(
            VerificationEvent::new_error_from_error(&e),
            format!("Error with VerifyShuffle for bb {}", context.bb_id),
        ),
    };

    input
        .c_mix_5
        .iter()
        .enumerate()
        .zip(input.ms.iter())
        .zip(input.pi_dec_5.iter())
        .map(|(((i, c_mix_5_i), m_i), pi_dec_5_i)| {
            let mut res = VerificationResult::new();
            match verify_decryption(
                context.eg,
                c_mix_5_i,
                eb_pk_cut.as_slice(),
                m_i,
                i_aux.as_slice(),
                (&pi_dec_5_i.e, pi_dec_5_i.z.as_slice()),
            ) {
                Ok(r) => {
                    if !r {
                        res.push_with_context(
                            VerificationEvent::new_failure(&r.to_string()),
                            format!(
                                "VerifyDecrpyption at position {} for bb {}",
                                i, context.bb_id
                            ),
                        );
                    }
                }
                Err(e) => res.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "Error with VerifyDecrpyption at position {} for bb {}",
                    i, context.bb_id
                ))),
            };
            res
        })
        .for_each(|mut r| res.append(&mut r));

    let context_43 = ContextAlgorithm43 {
        eg: context.eg,
        p_table: context.p_table,
    };
    let input_43 = InputsAlgorithm43 {
        ms: input.ms,
        upper_l_votes: input.upper_l_votes,
        upper_l_decoded_votes: input.upper_l_decoded_votes,
        upper_l_write_ins: input.upper_l_write_ins,
    };
    res.append_with_context(
        &verify_process_plaintexts(&context_43, &input_43),
        "VerifyProcessPlaintexts",
    );

    res
}
