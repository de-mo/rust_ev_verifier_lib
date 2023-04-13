use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::VerificationDirectory,
};

pub(super) fn fn_verification(dir: &VerificationDirectory, result: &mut VerificationResult) {
    let setup_dir = dir.unwrap_setup();
    let vcs_contexts = match setup_dir.election_event_context_payload() {
        Ok(o) => o.election_event_context.verification_card_set_contexts,
        Err(e) => {
            result.push_error(create_verification_error!(
                "Cannot extract election_event_context_payload",
                e
            ));
            return;
        }
    };
    let total_voter = match setup_dir.election_event_configuration() {
        Ok(o) => o.voter_total,
        Err(e) => {
            result.push_error(create_verification_error!(
                "Cannot extract election_event_context_payload",
                e
            ));
            return;
        }
    };
    if total_voter
        != vcs_contexts
            .iter()
            .map(|e| e.number_of_voting_cards)
            .sum::<usize>()
    {
        result.push_failure(create_verification_failure!(format!(
            "The sum of voting cards is not the same as total voters {}",
            total_voter
        )))
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
