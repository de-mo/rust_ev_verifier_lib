use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ConstantsTrait, Integer};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let ee_context = match context_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
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
    let p_prime: Vec<usize> = ee_context
        .small_primes
        .iter()
        .take(p_tilde.len())
        .copied()
        .collect();
    if p_prime != p_tilde {
        result.push(VerificationEvent::new_failure(
            "VerifA: prime group members and encoding voting options are not the same",
        ))
    }
    let verifb = ee_context
        .small_primes
        .iter()
        .take(Config::maximum_supported_number_of_selections_psi_sup() - 1)
        .skip(
            Config::maximum_number_of_supported_voting_options_n_sup()
                - Config::maximum_supported_number_of_selections_psi_sup()
                - 1,
        )
        .fold(Integer::zero().clone(), |acc, e| acc * e);
    if &verifb >= ee_context.encryption_group.p() {
        result.push(VerificationEvent::new_failure(
            "VerifB: The product of the phi last primes (the largest possible encoded vote) must be smaller than p"
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
