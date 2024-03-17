use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{setup_directory::SetupDirectoryTrait, VerificationDirectoryTrait},
};
use anyhow::anyhow;
use log::debug;
use rust_ev_crypto_primitives::EncryptionParameters;

pub(super) fn fn_0501_verify_encryption_parameters<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg: Box<
        crate::data_structures::setup::election_event_context_payload::ElectionEventContextPayload,
    > = match setup_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(create_verification_error!(
                "election_event_context_payload cannot be read",
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
            eg.encryption_group.p(),
            eg_test.p()
        )));
    }
    if eg_test.q() != eg.encryption_group.q() {
        result.push(create_verification_failure!(format!(
            "payload q and calculated q are equal: payload: {} / calculated: {}",
            eg.encryption_group.q(),
            eg_test.q()
        )));
    }
    if eg_test.g() != eg.encryption_group.g() {
        result.push(create_verification_failure!(format!(
            "payload g and calculated g are equal: payload: {} / calculated: {}",
            eg.encryption_group.g(),
            eg_test.g()
        )))
    }
}

pub(super) fn fn_0502_verify_small_prime_group_members<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(create_verification_error!(
                "election_event_context_payload cannot be read",
                e
            ));
            return;
        }
    };
    let primes = match eg
        .encryption_group
        .get_small_prime_group_members(Config::maximum_number_of_voting_options())
    {
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
        let mut i = 0usize;
        while eg.small_primes[i] == primes[i] {
            i += 1;
        }
        result.push(
            create_verification_failure!(
                format!(
                    "Small prime group members are not the same. First error at position {}: calculated {} / expected {}",
                    i + 1,
                    primes[i],
                    eg.small_primes[i]
                )
            )
        )
    }
}

#[cfg(test)]
mod test {
    use super::{super::super::super::result::VerificationResultTrait, *};
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    #[ignore]
    fn test_0501_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0501_verify_encryption_parameters(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0502_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0502_verify_small_prime_group_members(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok().unwrap() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok().unwrap());
    }
}
