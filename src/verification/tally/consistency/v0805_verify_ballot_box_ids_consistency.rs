use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{
        tally_directory::BBDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait,
    },
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();

    for bb_dir in tally_dir.bb_directories().iter() {
        result.append_with_context(
            &verify_for_bb_directory(bb_dir),
            format!("Ballot box directory {}", bb_dir.name()),
        );
    }
}

fn verify_for_bb_directory<B: BBDirectoryTrait>(bb_dir: &B) -> VerificationResult {
    let mut result = VerificationResult::new();

    let bb_id = bb_dir.name();

    for (i, cc_bb_payload_res) in bb_dir.control_component_ballot_box_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => {
                if p.ballot_box_id != bb_id {
                    result.push(VerificationEvent::new_failure(&format!(
                    "bb_id (={}) in {}/control_component_ballot_box_payload_{} is not the same than the directory",
                    &p.ballot_box_id, bb_id, i
                )));
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_ballot_box_payload_{} cannot be read",
                bb_id, i
            ))),
        }
    }

    for (i, cc_bb_payload_res) in bb_dir.control_component_shuffle_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => {
                if p.ballot_box_id != bb_id {
                    result.push(VerificationEvent::new_failure(&format!(
                    "bb_id (={}) in {}/control_component_shuffle_payload_{} is not the same than the directory",
                    &p.ballot_box_id, bb_id, i
                )));
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_shuffle_payload_{} cannot be read",
                bb_id, i
            ))),
        }
    }

    match bb_dir.tally_component_shuffle_payload() {
        Ok(p) => {
            if p.ballot_box_id != bb_id {
                result.push(VerificationEvent::new_failure(&format!(
                "bb_id (={}) in {}/tally_component_shuffle_payload is not the same than the directory",
                &p.ballot_box_id, bb_id
            )));
            }
        }
        Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
            "{}/tally_component_shuffle_payload cannot be read",
            bb_id
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
