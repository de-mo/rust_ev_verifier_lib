use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{
        tally_directory::BBDirectoryTrait, ContextDirectoryTrait, TallyDirectoryTrait,
        VerificationDirectoryTrait,
    },
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let tally_dir = dir.unwrap_tally();

    let ee_id = match context_dir.election_event_context_payload() {
        Ok(p) => p.election_event_context.election_event_id,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };

    for bb_dir in tally_dir.bb_directories().iter() {
        result.append_with_context(
            &verify_for_bb_directory(bb_dir, &ee_id),
            format!("Ballot box directory {}", bb_dir.name()),
        );
    }
}

fn test_election_event_id(ee_id: &str, expected: &str) -> VerificationResult {
    let mut result = VerificationResult::new();
    if ee_id != expected {
        result.push(VerificationEvent::new_failure(&format!(
            "Election Event ID {} not equal to {}",
            ee_id, expected
        )));
    }
    result
}

fn verify_for_bb_directory<B: BBDirectoryTrait>(bb_dir: &B, ee_id: &str) -> VerificationResult {
    let mut result = VerificationResult::new();

    let bb_name = bb_dir.name();

    for (i, cc_bb_payload_res) in bb_dir.control_component_ballot_box_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => result.append_with_context(
                &test_election_event_id(&p.election_event_id, ee_id),
                format!("{}/control_component_ballot_box_payload_{}", bb_name, i),
            ),
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_ballot_box_payload_{} cannot be read",
                bb_name, i
            ))),
        }
    }

    for (i, cc_bb_payload_res) in bb_dir.control_component_shuffle_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => result.append_with_context(
                &test_election_event_id(&p.election_event_id, ee_id),
                format!("{}/control_component_shuffle_payload_{}", bb_name, i),
            ),
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_shuffle_payload_{} cannot be read",
                bb_name, i
            ))),
        }
    }

    match bb_dir.tally_component_votes_payload() {
        Ok(p) => result.append_with_context(
            &test_election_event_id(&p.election_event_id, ee_id),
            format!("{}/tally_component_votes_payload", bb_name),
        ),
        Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
            "{}/tally_component_shuffle_payload cannot be read",
            bb_name
        ))),
    }

    match bb_dir.tally_component_shuffle_payload() {
        Ok(p) => result.append_with_context(
            &test_election_event_id(&p.election_event_id, ee_id),
            format!("{}/tally_component_shuffle_payload", bb_name),
        ),
        Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
            "{}/tally_component_shuffle_payload cannot be read",
            bb_name
        ))),
    }

    result
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
