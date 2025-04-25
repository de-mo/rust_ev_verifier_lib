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

use crate::verification::{VerificationEvent, VerificationResult};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, Integer,
};
use rust_ev_system_library::{
    preliminaries::{EPPTableAsContext, PTable},
    tally_phase::mix_offline::ProcessPlaintextsOutput,
};

pub struct ContextAlgorithm43<'a> {
    pub eg: &'a EncryptionParameters,
    pub p_table: &'a PTable,
}

pub struct InputsAlgorithm43<'a> {
    pub ms: &'a [&'a [Integer]],
    pub upper_l_votes: &'a [Vec<usize>],
    pub upper_l_decoded_votes: &'a [Vec<String>],
    pub upper_l_write_ins: &'a [Vec<String>],
}

impl<'a> From<&'a ContextAlgorithm43<'a>> for EPPTableAsContext<'a, 'a> {
    fn from(value: &'a ContextAlgorithm43<'a>) -> Self {
        Self::new(value.eg, value.p_table)
    }
}

pub fn verify_process_plaintexts(
    context: &ContextAlgorithm43,
    input: &InputsAlgorithm43,
) -> VerificationResult {
    let process_output_res =
        ProcessPlaintextsOutput::process_plaintexts(&EPPTableAsContext::from(context), input.ms);
    if let Err(e) = process_output_res {
        return VerificationResult::from(
            &VerificationEvent::new_error(&e).add_context("Error VerifyProcessPlaintexts"),
        );
    }
    let process_output = process_output_res.unwrap();
    let mut res = VerificationResult::new();
    if input.upper_l_votes != process_output.l_votes {
        res.push(VerificationEvent::new_failure(
            "Selected encoded voting options not the same",
        ));
    }
    if input.upper_l_decoded_votes != process_output.l_decoded_votes {
        res.push(VerificationEvent::new_failure(
            "Selected decoded voting options not the same",
        ));
    }
    if input.upper_l_write_ins != process_output.l_write_ins {
        res.push(VerificationEvent::new_failure(
            "Selected decoded write_ins not the same",
        ));
    }
    res
}
