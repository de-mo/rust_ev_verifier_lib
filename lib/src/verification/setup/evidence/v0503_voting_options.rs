use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{setup_directory::SetupDirectoryTrait, VerificationDirectoryTrait},
};
use anyhow::anyhow;
use crypto_primitives::num_bigint::Constants;
use log::debug;
use num_bigint::BigUint;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
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
    let ee_context = match setup_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(create_verification_error!(
                "election_event_context_payload cannot be read",
                e
            ));
            return;
        }
    };
    let mut p_tilde = vec![];
    for vcsc in ee_context
        .election_event_context
        .verification_card_set_contexts
        .iter()
    {
        p_tilde.extend(
            vcsc.primes_mapping_table
                .p_table
                .iter()
                .map(|e| e.encoded_voting_option),
        );
    }
    p_tilde.sort(); // Sort the primes
    p_tilde.dedup(); // remove duplicates
    let p_prime: Vec<usize> = eg
        .small_primes
        .iter()
        .take(p_tilde.len())
        .copied()
        .collect();
    if p_prime != p_tilde {
        result.push(create_verification_failure!(
            "VerifA: prime group members and encoding voting options are not the same"
        ))
    }
    let mut verifb: BigUint = BigUint::zero();
    for i in (Config::maximum_number_of_voting_options()
        - Config::maximum_number_of_selectable_voting_options())
        ..Config::maximum_number_of_selectable_voting_options()
    {
        verifb = &verifb * BigUint::from(eg.small_primes[i]);
    }
    if verifb >= eg.encryption_group.p {
        result.push(create_verification_failure!(
            "VerifB: The product of the phi last primes (the largest possible encoded vote) must be smaller than p"
        ))
    }
}

#[cfg(test)]
mod test {
    use super::{super::super::super::result::VerificationResultTrait, *};
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
