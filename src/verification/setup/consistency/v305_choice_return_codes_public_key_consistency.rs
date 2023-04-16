use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    crypto_primitives::num_bigint::{Constants, Operations},
    error::{create_verifier_error, VerifierError},
    file_structure::{
        setup_directory::{SetupDirectoryTrait, VCSDirectoryTrait},
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use num_bigint::BigUint;

pub(super) fn fn_verification<
    B: BBDirectoryTrait,
    V: VCSDirectoryTrait,
    S: SetupDirectoryTrait<V>,
    T: TallyDirectoryTrait<B>,
>(
    dir: &dyn VerificationDirectoryTrait<B, V, S, T>,
    result: &mut VerificationResult,
) {
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
    let setup_ccr = sc_pk
        .setup_component_public_keys
        .choice_return_codes_encryption_public_key;

    for (i, ccr) in setup_ccr.iter().enumerate() {
        let product_ccr = combined_cc_pk
            .iter()
            .map(|e| &e.ccrj_choice_return_codes_encryption_public_key[i])
            .fold(BigUint::one(), |acc, x| acc.mod_multiply(x, &eg_p));
        if &product_ccr != ccr {
            result.push_failure(create_verification_failure!(format!(
                "The ccr at position {} is not the product of the cc ccr",
                i
            )));
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::super::{verification::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::file_structure::VerificationDirectory;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
