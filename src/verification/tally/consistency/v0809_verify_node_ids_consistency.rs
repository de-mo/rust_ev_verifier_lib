use std::collections::HashSet;

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{
        tally_directory::BBDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait,
    },
};

const LIST_CC_NUMBER: &[usize] = &[1, 2, 3, 4];

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

fn verifiy_one_to_for(list: &[usize]) -> VerificationResult {
    let mut result = VerificationResult::new();
    if list.iter().collect::<HashSet<_>>() != LIST_CC_NUMBER.iter().collect::<HashSet<_>>() {
        result.push(VerificationEvent::new_failure(&format!(
            "The list of node ids (={:?}) does not correspond to the expected list (={:?})",
            list, LIST_CC_NUMBER
        )))
    }
    result
}

fn verify_for_bb_directory<B: BBDirectoryTrait>(bb_dir: &B) -> VerificationResult {
    let mut result = VerificationResult::new();

    let bb_name = bb_dir.name();

    result.append_with_context(
        &verifiy_one_to_for(
            bb_dir
                .control_component_ballot_box_payload_group()
                .get_numbers()
                .as_slice(),
        ),
        format!("{}/control_component_ballot_box_payload", bb_name),
    );

    result.append_with_context(
        &verifiy_one_to_for(
            bb_dir
                .control_component_shuffle_payload_group()
                .get_numbers()
                .as_slice(),
        ),
        format!("{}/control_component_shuffle_payload", bb_name),
    );

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
