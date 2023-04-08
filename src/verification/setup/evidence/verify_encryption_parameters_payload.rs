use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationError,
        VerificationErrorType, VerificationFailure, VerificationFailureType,
    },
    verification::{Verification, VerificationMetaData},
    VerificationCategory, VerificationPeriod,
};
use crate::{
    crypto_primitives::{
        elgamal::{get_encryption_parameters, get_small_prime_group_members},
        MAX_NB_SMALL_PRIMES,
    },
    error::{create_verifier_error, VerifierError},
    file_structure::VerificationDirectory,
};

pub(super) fn get_verification_500() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "500".to_owned(),
            nr: "5.01".to_owned(),
            name: "VerifyEncryptionParameters".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Evidence,
        },
        fn_verification_500,
    )
}

pub(super) fn get_verification_501() -> Verification {
    Verification::new(
        VerificationMetaData {
            id: "501".to_owned(),
            nr: "5.02".to_owned(),
            name: "VerifySmallPrimeGroupMembers".to_owned(),
            period: VerificationPeriod::Setup,
            category: VerificationCategory::Evidence,
        },
        fn_verification_501,
    )
}

fn fn_verification_500(
    dir: &VerificationDirectory,
) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut failures: Vec<VerificationFailure> = vec![];
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(eg) => eg,
        Err(e) => {
            return (
                vec![create_verification_error!(
                    "encryption_parameters_payload cannot be read",
                    e
                )],
                failures,
            )
        }
    };
    let eg_test = match get_encryption_parameters(&eg.seed) {
        Ok(eg) => eg,
        Err(e) => {
            return (
                vec![create_verification_error!(
                    "Error getting encrpytion parameters",
                    e
                )],
                failures,
            )
        }
    };
    if eg_test.p != eg.encryption_group.p {
        failures.push(create_verification_failure!("p are equal in {}"))
    }
    if eg_test.q != eg.encryption_group.q {
        failures.push(create_verification_failure!("q are equal in {}"))
    }
    if eg_test.g != eg.encryption_group.g {
        failures.push(create_verification_failure!("g are equal in {}"))
    }
    (vec![], failures)
}

fn fn_verification_501(
    dir: &VerificationDirectory,
) -> (Vec<VerificationError>, Vec<VerificationFailure>) {
    let mut failures: Vec<VerificationFailure> = vec![];
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(eg) => eg,
        Err(e) => {
            return (
                vec![create_verification_error!(
                    "encryption_parameters_payload cannot be read",
                    e
                )],
                failures,
            )
        }
    };
    let primes = match get_small_prime_group_members(&eg.encryption_group.p, MAX_NB_SMALL_PRIMES) {
        Ok(p) => p,
        Err(e) => {
            return (
                vec![create_verification_error!(
                    "Error getting small prime group members",
                    e
                )],
                failures,
            )
        }
    };
    if eg.small_primes.len() != primes.len() {
        failures.push(create_verification_failure!(format!(
            "length of primes not the same: calculated: {} / expected {}",
            primes.len(),
            eg.small_primes.len()
        )))
    } else {
        if eg.small_primes != primes {
            failures.push(create_verification_failure!(
                "Small prime group members are not the same"
            ))
        }
    }
    (vec![], failures)
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
    fn test_500_ok() {
        let dir = get_verifier_dir();
        let (e, f) = fn_verification_500(&dir);
        assert!(e.is_empty());
        assert!(f.is_empty());
    }

    #[test]
    fn test_501_ok() {
        let dir = get_verifier_dir();
        let (e, f) = fn_verification_501(&dir);
        assert!(e.is_empty());
        assert!(f.is_empty());
    }
}
