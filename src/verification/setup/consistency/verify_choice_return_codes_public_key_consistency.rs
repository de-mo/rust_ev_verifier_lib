use crate::file_structure::VerificationDirectory;
use crate::{
    crypto_primitives::num_bigint::Constants,
    error::{create_verifier_error, VerifierError},
};
use num_bigint::BigUint;

use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::{Verification, VerificationMetaData, VerificationResult},
    VerificationCategory, VerificationPeriod,
};

pub(super) fn get_verification_305() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "305".to_owned(),
            nr: "3.06".to_owned(),
            name: "VerifyChoiceReturnCodesPublicKeyConsistency".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Consistency,
        },
        fn_verification_305,
    )
}

fn fn_verification_305(dir: &VerificationDirectory, result: &mut VerificationResult) {
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
    let setup_ccr = sc_pk
        .setup_component_public_keys
        .choice_return_codes_encryption_public_key;
    for (i, ccr) in setup_ccr.iter().enumerate() {
        let product_ccr = sc_pk
            .setup_component_public_keys
            .combined_control_component_public_keys
            .iter()
            .map(|e| &e.ccrj_choice_return_codes_encryption_public_key[i])
            .fold(BigUint::one(), |acc, x| acc * x);
        let calculated_ccr = product_ccr % &eg_p;
        if &calculated_ccr != ccr {
            result.push_failure(create_verification_failure!(format!(
                "The ccr at position {} is not the product of the cc ccr",
                i
            )));
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::super::super::verification::VerificationResultTrait;
    use crate::file_structure::setup_directory::SetupDirectory;

    use super::*;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::Setup(SetupDirectory::new(&location))
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_305(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
