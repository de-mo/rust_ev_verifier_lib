use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};
use anyhow::anyhow;
use log::debug;
use rug::Integer;
use rust_ev_crypto_primitives::{Constants, Operations};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let eg = match context_dir.election_event_context_payload() {
        Ok(o) => o.encryption_group,
        Err(e) => {
            result.push(create_verification_error!(
                "Cannot extract election_event_context_payload",
                e
            ));
            return;
        }
    };
    let sc_pk = match context_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(create_verification_error!(
                "Cannot extract setup_component_public_keys_payload",
                e
            ));
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
            result.push(create_verification_failure!(format!(
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
