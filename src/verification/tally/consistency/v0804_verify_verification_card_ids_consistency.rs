use std::collections::HashSet;

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{
        context_directory::ContextVCSDirectoryTrait, tally_directory::BBDirectoryTrait,
        ContextDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait,
    },
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
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

    for vcs_dir in context_dir.vcs_directories().iter() {
        let vcs_id = vcs_dir.name();
        let payload = match vcs_dir.setup_component_tally_data_payload() {
            Ok(p) => p,
            Err(e) => {
                result.push(
                    VerificationEvent::new_error(&e)
                        .add_context("setup_component_tally_data_payload cannot be read"),
                );
                return;
            }
        };
        let hs_vc_ids = payload
            .verification_card_ids
            .iter()
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();
        let bb_id = match ee_context.get_ballot_box_id(vcs_id.as_str()) {
            Some(id) => id,
            None => {
                result.push(VerificationEvent::new_error(&format!(
                    "ballot box id for vcs_id {} not found in setup_component_tally_data_payload",
                    vcs_id
                )));
                return;
            }
        };
        let bb_dir = match tally_dir
            .bb_directories()
            .iter()
            .find(|p| p.name() == bb_id)
        {
            Some(p) => p,
            None => {
                result.push(VerificationEvent::new_error(&format!(
                    "ballot box for bb_id {} not found in the ballot box directories",
                    bb_id
                )));
                return;
            }
        };
        for (i, cc_bb_paylod) in bb_dir.control_component_ballot_box_payload_iter() {
            if let Err(e) = cc_bb_paylod {
                result.push(VerificationEvent::new_error(&e).add_context(format!(
                    "{}/control_component_ballot_box_payload_{} cannot be read",
                    bb_id, i
                )));
                break;
            }
            let bb_vc_ids = cc_bb_paylod
                .as_ref()
                .unwrap()
                .confirmed_encrypted_votes
                .iter()
                .map(|v| v.context_ids.verification_card_id.as_str())
                .collect::<Vec<_>>();
            if !bb_vc_ids.iter().all(|id| hs_vc_ids.contains(id)) {
                result.push(VerificationEvent::new_failure(&format!(
                    "The voting card ids in {}/control_component_ballot_box_payload_{} are not a subset of the vc ids in {}/setup_component_tally_data_payload",
                    bb_id, i, vcs_id
                )));
                return;
            }
        }
    }
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
