use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{setup_directory::SetupDirectoryTrait, VerificationDirectoryTrait},
};
use anyhow::anyhow;
use log::debug;

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
        .get_small_prime_group_members(Config::maximum_number_of_supported_voting_options_n_sup())
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
