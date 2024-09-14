use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};

use rust_ev_crypto_primitives::EncryptionParameters;

pub(super) fn fn_0501_verify_encryption_parameters<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
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
