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
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ConstantsTrait, OperationsTrait};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
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
    let eg = &context.as_ref().encryption_group;
    let sc_pk = match context_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot extract setup_component_public_keys_payload"),
            );
            return;
        }
    };
    let combined_cc_pk = &sc_pk
        .setup_component_public_keys
        .combined_control_component_public_keys;
    let setup_el_pk = &sc_pk.setup_component_public_keys.election_public_key;

    for (i, el_pk_i) in setup_el_pk.iter().enumerate() {
        let product_cc_el_pk = combined_cc_pk
            .iter()
            .map(|e| &e.ccmj_election_public_key[i])
            .fold(Integer::one().clone(), |acc, x| acc.mod_multiply(x, eg.p()));
        let calculated_el_pk = product_cc_el_pk.mod_multiply(
            &sc_pk.setup_component_public_keys.electoral_board_public_key[i],
            eg.p(),
        );
        if &calculated_el_pk != el_pk_i {
            result.push(VerificationEvent::new_failure(&format!(
                "The election public key EL_pk at {} is correctly combined",
                i
            )));
        }
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
