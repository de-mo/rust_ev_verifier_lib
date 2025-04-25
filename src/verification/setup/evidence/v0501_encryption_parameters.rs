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

use rust_ev_system_library::rust_ev_crypto_primitives::prelude::elgamal::EncryptionParameters;

pub(super) fn fn_0501_verify_encryption_parameters<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let eg = match context_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    let eg_test = match EncryptionParameters::get_encryption_parameters(&eg.seed) {
        Ok(eg) => eg,
        Err(e) => {
            result.push(VerificationEvent::new_error(&e).add_context(format!(
                "Error calculating encrpytion parameters from seed {}",
                eg.seed
            )));
            return;
        }
    };
    if eg_test.p() != eg.encryption_group.p() {
        result.push(VerificationEvent::new_failure(&format!(
            "payload p and calculated p are not equal: payload: {} / calculated: {}",
            eg.encryption_group.p(),
            eg_test.p()
        )));
    }
    if eg_test.q() != eg.encryption_group.q() {
        result.push(VerificationEvent::new_failure(&format!(
            "payload q and calculated q are not equal: payload: {} / calculated: {}",
            eg.encryption_group.q(),
            eg_test.q()
        )));
    }
    if eg_test.g() != eg.encryption_group.g() {
        result.push(VerificationEvent::new_failure(&format!(
            "payload g and calculated g are not equal: payload: {} / calculated: {}",
            eg.encryption_group.g(),
            eg_test.g()
        )))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_0501_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0501_verify_encryption_parameters(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
