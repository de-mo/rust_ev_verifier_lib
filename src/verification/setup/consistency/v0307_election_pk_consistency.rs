use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{setup_directory::SetupDirectoryTrait, VerificationDirectoryTrait},
};
use anyhow::anyhow;
use log::debug;
use num_bigint::BigUint;
use rust_ev_crypto_primitives::num_bigint::{Constants, Operations};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg_p = match setup_dir.encryption_parameters_payload() {
        Ok(o) => o.encryption_group.p,
        Err(e) => {
            result.push(create_verification_error!(
                "Cannot extract encryption_parameters_payload",
                e
            ));
            return;
        }
    };
    let sc_pk = match setup_dir.setup_component_public_keys_payload() {
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
    let setup_el_pk = sc_pk.setup_component_public_keys.election_public_key;

    for (i, el_pk_i) in setup_el_pk.iter().enumerate() {
        let product_cc_el_pk = combined_cc_pk
            .iter()
            .map(|e| &e.ccmj_election_public_key[i])
            .fold(BigUint::one(), |acc, x| acc.mod_multiply(x, &eg_p));
        let calculated_el_pk = product_cc_el_pk.mod_multiply(
            &sc_pk.setup_component_public_keys.electoral_board_public_key[i],
            &eg_p,
        );
        if &calculated_el_pk != el_pk_i {
            result.push(create_verification_failure!(format!(
                "The election public key EL_pk at {} is correctly combined",
                i
            )));
        }
    }
}

#[cfg(test)]
mod test {
    use super::{super::super::super::result::VerificationResultTrait, *};
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
