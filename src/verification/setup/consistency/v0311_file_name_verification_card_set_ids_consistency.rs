use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{
        context_directory::ContextDirectoryTrait, setup_directory::SetupDirectoryTrait,
        VerificationDirectoryTrait,
    },
};

fn verify_file_name_correct(vcs_ids: &[&str], dir_names: &[String]) -> VerificationResult {
    let mut res = VerificationResult::new();
    let mut vcs_ids_ordered = vcs_ids.to_vec();
    vcs_ids_ordered.sort();
    let mut dir_names_ordered = dir_names.to_vec();
    dir_names_ordered.sort();
    if vcs_ids_ordered != dir_names_ordered {
        res.push(VerificationEvent::new_failure(&format!(
            "The subdirectory [{}] are not equal to the list of voting card set ids [{}]",
            dir_names_ordered.join(","),
            vcs_ids_ordered.join(",")
        )))
    }
    res
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_dir = dir.unwrap_setup();

    let ee_context = match context_dir.election_event_context_payload() {
        Ok(payload) => payload,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot read payload for election_event_context_payload"),
            );
            return;
        }
    };
    let vcs_ids = ee_context.election_event_context.vcs_ids();

    result.append_with_context(
        &verify_file_name_correct(&vcs_ids, &context_dir.vcs_directory_names()),
        "Context directory",
    );

    result.append_with_context(
        &verify_file_name_correct(&vcs_ids, &setup_dir.vcs_directory_names()),
        "Setup directory",
    );
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        get_test_verifier_mock_setup_dir as get_mock_verifier_dir,
        get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST,
    };

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{}", e);
            }
            for f in result.failures() {
                println!("{}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_nok_add_vcs() {
        let mut dir = get_mock_verifier_dir();
        let mut election_event_context_payload =
            dir.context().election_event_context_payload().unwrap();
        let mut context = election_event_context_payload
            .as_ref()
            .election_event_context
            .verification_card_set_contexts[0]
            .clone();
        context.verification_card_set_id = "toto".to_string();
        election_event_context_payload
            .as_mut()
            .election_event_context
            .verification_card_set_contexts
            .push(context);
        dir.context_mut()
            .mock_election_event_context_payload(&Ok(&election_event_context_payload));
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
        assert!(!result.has_errors());
    }

    #[test]
    fn test_nok_change_vcs_id() {
        let mut dir = get_mock_verifier_dir();
        let mut election_event_context_payload =
            dir.context().election_event_context_payload().unwrap();
        election_event_context_payload
            .as_mut()
            .election_event_context
            .verification_card_set_contexts[0]
            .verification_card_set_id = "toto".to_string();
        dir.context_mut()
            .mock_election_event_context_payload(&Ok(&election_event_context_payload));
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
        assert!(!result.has_errors());
    }

    #[test]
    fn test_nok_change_remove_vcs() {
        let mut dir = get_mock_verifier_dir();
        let mut election_event_context_payload =
            dir.context().election_event_context_payload().unwrap();
        election_event_context_payload
            .as_mut()
            .election_event_context
            .verification_card_set_contexts
            .pop();
        dir.context_mut()
            .mock_election_event_context_payload(&Ok(&election_event_context_payload));
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
        assert!(!result.has_errors());
    }
}
