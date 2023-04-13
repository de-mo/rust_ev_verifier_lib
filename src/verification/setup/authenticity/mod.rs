use super::super::{
    error::{create_verification_error, VerificationErrorType},
    verification::{Verification, VerificationResult},
    verification_suite::VerificationList,
    verify_signature_for_object,
};
use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::VerificationDirectory,
    verification::meta_data::VerificationMetaDataList,
};

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res = vec![];
    res.push(Verification::new("s200", fn_verification_200, metadata_list).unwrap());
    res.push(Verification::new("s202", fn_verification_202, metadata_list).unwrap());
    res.push(Verification::new("s203", fn_verification_203, metadata_list).unwrap());
    res
}

fn fn_verification_200(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push_error(create_verification_error!(
                format!("{} cannot be read", "encryption_parameters_payload"),
                e
            ));
            return;
        }
    };
    verify_signature_for_object(eg.as_ref(), result, "encryption_parameters_payload")
}

fn fn_verification_202(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.setup_component_public_keys_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push_error(create_verification_error!(
                format!("{} cannot be read", "setup_component_public_keys_payload"),
                e
            ));
            return;
        }
    };
    verify_signature_for_object(eg.as_ref(), result, "setup_component_public_keys_payload")
}

fn fn_verification_203(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    for (i, cc) in setup_dir.control_component_public_keys_payload_iter() {
        match cc {
            Ok(cc) => verify_signature_for_object(
                cc.as_ref(),
                result,
                &format!("control_component_public_keys_payload_{}", i),
            ),
            Err(e) => result.push_error(create_verification_error!(
                format!("control_component_public_keys_payload_{} cannot be read", i),
                e
            )),
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
    fn test_200() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_200(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_202() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_202(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_203() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_203(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
