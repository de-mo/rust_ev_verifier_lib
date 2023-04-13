use super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::{Verification, VerificationResult},
    verification_suite::VerificationList,
    verifiy_signature,
};
use crate::{
    constants::direct_trust_path,
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

verifiy_signature!(
    fn_verification_200,
    encryption_parameters_payload,
    "encryption_parameters_payload"
);

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
