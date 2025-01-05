use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ConstantsTrait, OperationsTrait};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let eg = match context_dir.election_event_context_payload() {
        Ok(o) => o.encryption_group,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot extract election_event_context_payload"),
            );
            return;
        }
    };
    let sc_pk = match context_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot extract setup_component_public_keys_payload"),
            );
            return;
        }
    };
    let combined_cc_pk = sc_pk
        .setup_component_public_keys
        .combined_control_component_public_keys;
    let setup_ccr = sc_pk
        .setup_component_public_keys
        .choice_return_codes_encryption_public_key;

    for (i, ccr) in setup_ccr.iter().enumerate() {
        let product_ccr = combined_cc_pk
            .iter()
            .map(|e| &e.ccrj_choice_return_codes_encryption_public_key[i])
            .fold(Integer::one().clone(), |acc, x| acc.mod_multiply(x, eg.p()));
        if &product_ccr != ccr {
            result.push(VerificationEvent::new_failure(&format!(
                "The ccr at position {} is not the product of the cc ccr",
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
