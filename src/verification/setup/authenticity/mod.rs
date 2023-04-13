use std::path::Path;

use super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::{Verification, VerificationResult},
    verification_suite::VerificationList,
};
use crate::{
    crypto_primitives::signature::VerifiySignatureTrait,
    error::{create_verifier_error, VerifierError},
    file_structure::VerificationDirectory,
    verification::meta_data::VerificationMetaDataList,
};

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res = vec![];
    res.push(Verification::new("s200", fn_verification_200, metadata_list).unwrap());
    res
}

fn fn_verification_200(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push_error(create_verification_error!(
                "encryption_parameters_payload cannot be read",
                e
            ));
            return;
        }
    };
    match eg
        .as_ref()
        .verifiy_signature(&Path::new(".").join("datasets").join("direct-trust"))
    {
        Ok(t) => {
            if !t {
                result.push_failure(create_verification_failure!(
                    "Wrong signature for encryption_parameters_payload"
                ))
            }
        }
        Err(e) => {
            result.push_error(create_verification_error!(
                "Error testing signature of encryption_parameters_payload",
                e
            ));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::verification::VerificationPeriod;

    use super::super::super::verification::VerificationResultTrait;
    use super::*;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_200(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
