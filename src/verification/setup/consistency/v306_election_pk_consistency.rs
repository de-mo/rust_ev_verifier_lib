use crate::file_structure::VerificationDirectory;
use crate::{
    crypto_primitives::num_bigint::{Constants, Operations},
    error::{create_verifier_error, VerifierError},
};
use num_bigint::BigUint;

use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};

pub(super) fn fn_verification(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    let eg_p = match setup_dir.encryption_parameters_payload() {
        Ok(o) => o.encryption_group.p,
        Err(e) => {
            result.push_error(create_verification_error!(
                "Cannot extract encryption_parameters_payload",
                e
            ));
            return;
        }
    };
    let sc_pk = match setup_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push_error(create_verification_error!(
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
            result.push_failure(create_verification_failure!(format!(
                "The election public key EL_pk at {} is correctly combined",
                i
            )));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::verification::VerificationPeriod;

    use super::super::super::super::verification::VerificationResultTrait;
    use super::*;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::new(VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
