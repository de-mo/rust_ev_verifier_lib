use num_bigint::BigUint;

use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    constants::{MAXIMUM_NUMBER_OF_SELECTABLE_VOTING_OPTIONS, MAXIMUM_NUMBER_OF_VOTING_OPTIONS},
    crypto_primitives::num_bigint::Constants,
    error::{create_verifier_error, VerifierError},
    file_structure::VerificationDirectory,
};

pub(super) fn fn_verification(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push_error(create_verification_error!(
                "encryption_parameters_payload cannot be read",
                e
            ));
            return;
        }
    };
    let ee_context = match setup_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push_error(create_verification_error!(
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
                .map(|e| e.encoded_voting_option.clone()),
        );
    }
    p_tilde.sort(); // Sort the primes
    p_tilde.dedup(); // remove duplicates
    let p_prime: Vec<usize> = eg
        .small_primes
        .iter()
        .take(p_tilde.len())
        .map(|e| e.clone())
        .collect();
    if p_prime != p_tilde {
        result.push_failure(create_verification_failure!(
            "VerifA: prime group members and encoding voting options are not the same"
        ))
    }
    let mut verifb: BigUint = BigUint::zero();
    for i in (MAXIMUM_NUMBER_OF_VOTING_OPTIONS - MAXIMUM_NUMBER_OF_SELECTABLE_VOTING_OPTIONS)
        ..MAXIMUM_NUMBER_OF_VOTING_OPTIONS
    {
        verifb = &verifb * BigUint::from(eg.small_primes[i]);
    }
    if verifb >= eg.encryption_group.p {
        result.push_failure(create_verification_failure!(
            "VerifB: The product of the phi last primes (the largest possible encoded vote) must be smaller than p"
        ))
    }
}

#[cfg(test)]
mod test {

    use crate::verification::VerificationPeriod;

    use super::super::super::super::verification::VerificationResultTrait;
    use super::*;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset-setup1");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
