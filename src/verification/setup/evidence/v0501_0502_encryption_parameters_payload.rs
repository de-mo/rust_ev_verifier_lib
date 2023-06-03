use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    constants::MAXIMUM_NUMBER_OF_VOTING_OPTIONS,
    crypto_primitives::elgamal::{get_encryption_parameters, get_small_prime_group_members},
    file_structure::{setup_directory::SetupDirectoryTrait, VerificationDirectoryTrait},
};
use anyhow::anyhow;
use log::debug;

pub(super) fn fn_verification_0501<D: VerificationDirectoryTrait>(
    dir: &D,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(create_verification_error!(
                "encryption_parameters_payload cannot be read",
                e
            ));
            return;
        }
    };
    let eg_test = match get_encryption_parameters(&eg.seed) {
        Ok(eg) => eg,
        Err(e) => {
            result.push(create_verification_error!(
                format!(
                    "Error calculating encrpytion parameters from seed {}",
                    eg.seed
                ),
                e
            ));
            return;
        }
    };
    if eg_test.p != eg.encryption_group.p {
        result.push(create_verification_failure!(format!(
            "payload p and calculated p are equal: payload: {} / calculated: {}",
            eg.encryption_group.p, eg_test.p
        )))
    }
    if eg_test.q != eg.encryption_group.q {
        result.push(create_verification_failure!(format!(
            "payload q and calculated q are equal: payload: {} / calculated: {}",
            eg.encryption_group.q, eg_test.q
        )))
    }
    if eg_test.g != eg.encryption_group.g {
        result.push(create_verification_failure!(format!(
            "payload g and calculated g are equal: payload: {} / calculated: {}",
            eg.encryption_group.g, eg_test.g
        )))
    }
}

pub(super) fn fn_verification_0502<D: VerificationDirectoryTrait>(
    dir: &D,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(create_verification_error!(
                "encryption_parameters_payload cannot be read",
                e
            ));
            return;
        }
    };
    let primes = match get_small_prime_group_members(
        &eg.encryption_group.p,
        MAXIMUM_NUMBER_OF_VOTING_OPTIONS,
    ) {
        Ok(p) => p,
        Err(e) => {
            result.push(create_verification_error!(
                "Error getting small prime group members",
                e
            ));
            return;
        }
    };
    if eg.small_primes.len() != primes.len() {
        result.push(create_verification_failure!(format!(
            "length of primes not the same: calculated: {} / expected {}",
            primes.len(),
            eg.small_primes.len()
        )))
    } else {
        if eg.small_primes != primes {
            result.push(create_verification_failure!(
                "Small prime group members are not the same"
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::super::{result::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::file_structure::VerificationDirectory;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset1-setup-tally");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    #[ignore]
    fn test_500_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0501(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_501_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0502(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
