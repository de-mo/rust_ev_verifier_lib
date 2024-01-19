use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{setup_directory::SetupDirectoryTrait, VerificationDirectoryTrait},
};
use anyhow::anyhow;
use log::debug;
use rust_ev_crypto_primitives::{
    EncryptionParameters, get_small_prime_group_members,
};

pub(super) fn fn_verification_0501<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
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
    let eg_test = match EncryptionParameters::get_encryption_parameters(&eg.seed) {
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
    if eg_test.p() != eg.encryption_group.p() {
        result.push(create_verification_failure!(format!(
            "payload p and calculated p are equal: payload: {} / calculated: {}",
            eg.encryption_group.p(), eg_test.p()
        )))
    }
    if eg_test.q() != eg.encryption_group.q() {
        result.push(create_verification_failure!(format!(
            "payload q and calculated q are equal: payload: {} / calculated: {}",
            eg.encryption_group.q(), eg_test.q()
        )))
    }
    if eg_test.g() != eg.encryption_group.g() {
        result.push(create_verification_failure!(format!(
            "payload g and calculated g are equal: payload: {} / calculated: {}",
            eg.encryption_group.g(), eg_test.g()
        )))
    }
}

pub(super) fn fn_verification_0502<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
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
        eg.encryption_group.p(),
        Config::maximum_number_of_voting_options(),
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
    } else if eg.small_primes != primes {
        result.push(create_verification_failure!(
            "Small prime group members are not the same"
        ))
    }
}

#[cfg(test)]
mod test {
    use super::{super::super::super::result::VerificationResultTrait, *};
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    #[ignore]
    fn test_500_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0501(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_501_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0502(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
