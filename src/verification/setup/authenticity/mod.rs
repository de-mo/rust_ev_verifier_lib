use std::path::Path;

use crate::file_structure::VerificationDirectory;

use super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationError,
        VerificationErrorType, VerificationFailure, VerificationFailureType,
    },
    verification::{Verification, VerificationMetaData},
    VerificationCategory, VerificationList, VerificationPeriod,
};

use crate::{
    crypto_primitives::signature::VerifiySignatureTrait,
    error::{create_verifier_error, VerifierError},
};

pub fn get_verifications() -> VerificationList {
    let mut res = vec![];
    res.push(get_verification_200());
    res
}

pub(super) fn get_verification_200() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "200".to_owned(),
            nr: "2.01".to_owned(),
            name: "VerifySignatureSetupComponentEncryptionParameters".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Authenticity,
        },
        fn_verification_200,
    )
}

fn fn_verification_200(
    dir: &VerificationDirectory,
) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(p) => p,
        Err(e) => {
            return (
                vec![create_verification_error!(
                    "encryption_parameters_payload cannot be read",
                    e
                )],
                vec![],
            )
        }
    };
    match eg
        .as_ref()
        .verifiy_signature(&Path::new(".").join("datasets").join("direct-trust"))
    {
        Ok(t) => {
            if t {
                (vec![], vec![])
            } else {
                (
                    vec![],
                    vec![create_verification_failure!(
                        "Wrong signature for encryption_parameters_payload"
                    )],
                )
            }
        }
        Err(e) => (
            vec![create_verification_error!(
                "Error testing signature of encryption_parameters_payload",
                e
            )],
            vec![],
        ),
    }
}

#[cfg(test)]
mod test {
    use crate::file_structure::setup_directory::SetupDirectory;

    use super::*;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::Setup(SetupDirectory::new(&location))
    }

    #[test]
    #[ignore]
    fn test_ok() {
        let dir = get_verifier_dir();
        let (e, f) = fn_verification_200(&dir);
        assert!(e.is_empty());
        assert!(f.is_empty());
    }
}
