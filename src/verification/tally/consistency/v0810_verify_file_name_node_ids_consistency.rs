use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
};

fn verify_nod_ir_for_tally_bb_dir<B: BBDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    for (i, f) in dir.control_component_ballot_box_payload_iter() {
        match f {
            Ok(s) => {
                if s.node_id != i {
                    result.push_with_context(
                        VerificationEvent::new_failure(&format!("node id {} for control_component_ballot_box_payload.{} not same than index", s.node_id, i)),
                        format!("{}/control_component_ballot_box_payload.{}", dir.name(), i),
                    )
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_ballot_box_payload.{} has wrong format",
                dir.name(),
                i
            ))),
        }
    }

    for (i, f) in dir.control_component_shuffle_payload_iter() {
        match f {
            Ok(s) => {
                if s.node_id != i {
                    result.push_with_context(
                        VerificationEvent::new_failure(&format!("node id {} for control_component_shuffle_payload.{} not same than index", s.node_id, i)),
                        format!("{}/control_component_shuffle_payload.{}", dir.name(), i),
                    )
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_shuffle_payload.{} has wrong format",
                dir.name(),
                i
            ))),
        }
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();

    for bb in tally_dir.bb_directories().iter() {
        verify_nod_ir_for_tally_bb_dir(bb, result);
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
        assert!(result.is_ok());
    }
}
