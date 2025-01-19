use std::collections::HashSet;

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{ContextDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait},
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let tally_dir = dir.unwrap_tally();

    let ee_context = match context_dir.election_event_context_payload() {
        Ok(p) => p.election_event_context,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };

    let bb_ids = ee_context.bb_ids();
    let bb_dir_names = tally_dir.bb_directory_names();
    let hs_bb_dir_names = bb_dir_names
        .iter()
        .map(|e| e.as_str())
        .collect::<HashSet<_>>();

    bb_ids.iter().for_each(|id| if !hs_bb_dir_names.contains(id) {
        result.push(VerificationEvent::new_failure(&format!(
            "The ballot box id {} from election_event_context_payload is not a ballot box directory",
            id
        )))
    });

    hs_bb_dir_names.iter().for_each(|name| if !bb_ids.contains(name) {
        result.push(VerificationEvent::new_failure(&format!(
            "the ballot box directory {} is not in the list of ballot box ids from election_event_context_payload",
            name
        )))
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_tally_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for r in result.errors_to_string() {
                println!("{:?}", r)
            }
            for r in result.failures_to_string() {
                println!("{:?}", r)
            }
        }
        assert!(result.is_ok());
    }
}
