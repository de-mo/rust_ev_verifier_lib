use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let vcs_contexts = match context_dir.election_event_context_payload() {
        Ok(o) => o.election_event_context.verification_card_set_contexts,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot extract election_event_context_payload"),
            );
            return;
        }
    };
    let total_voter = match context_dir.election_event_configuration() {
        Ok(o) => match o.register.iter() {
            Ok(it) => it.count(),
            Err(e) => {
                result.push(
                    VerificationEvent::new_error(&e).add_context("Error iterating over the voters"),
                );
                return;
            }
        },
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot extract election_event_context_payload"),
            );
            return;
        }
    };
    if total_voter
        != vcs_contexts
            .iter()
            .map(|e| e.number_of_voting_cards)
            .sum::<usize>()
    {
        result.push(VerificationEvent::new_failure(&format!(
            "The sum of voting cards is not the same as total voters {}",
            total_voter
        )))
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
