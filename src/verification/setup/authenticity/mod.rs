use super::super::{
    error::{create_verification_error, VerificationErrorType},
    verification::{Verification, VerificationResult},
    verification_suite::VerificationList,
    verify_signature_for_object,
};
use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::{
        setup_directory::{SetupDirectoryTrait, VCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::meta_data::VerificationMetaDataList,
};
use log::debug;

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res = vec![];
    res.push(Verification::new("02.01", fn_verification_0201, metadata_list).unwrap());
    res.push(Verification::new("02.03", fn_verification_0203, metadata_list).unwrap());
    res.push(Verification::new("02.04", fn_verification_0204, metadata_list).unwrap());
    res.push(Verification::new("02.05", fn_verification_0205, metadata_list).unwrap());
    res
}

fn fn_verification_0201<D: VerificationDirectoryTrait>(dir: &D, result: &mut VerificationResult) {
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

fn fn_verification_0203<D: VerificationDirectoryTrait>(dir: &D, result: &mut VerificationResult) {
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

fn fn_verification_0204<D: VerificationDirectoryTrait>(dir: &D, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    for (i, cc) in setup_dir.control_component_public_keys_payload_iter() {
        debug!("Verification 2.04 for cc {}", i);
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

fn fn_verification_0205<D: VerificationDirectoryTrait>(dir: &D, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    for d in setup_dir.vcs_directories() {
        debug!("Verification 2.05 for vcs_dir {}", d.get_name());
        match setup_dir.setup_component_public_keys_payload() {
            Ok(p) => verify_signature_for_object(
                p.as_ref(),
                result,
                &format!("{}/setup_component_public_keys_payload", d.get_name()),
            ),
            Err(e) => result.push_error(create_verification_error!(
                format!(
                    "Error reading {}/setup_component_public_keys_payload",
                    d.get_name()
                ),
                e
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::{verification::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::file_structure::VerificationDirectory;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset1-setup-tally");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_0201() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0201(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0203() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0203(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0204() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0204(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0205() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0205(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
