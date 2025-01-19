use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{
        file_group::FileGroup,
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};

fn verify_uninterrupted_monotonic_sequence(
    fg: &FileGroup,
    result: &mut VerificationResult,
    dir: &String,
) {
    let mut numbers = fg.get_numbers().clone();
    numbers.sort();
    if !fg.has_elements() && numbers[0] + numbers[numbers.len() - 1] == numbers.len() {
        result.push(VerificationEvent::new_failure(&format!(
            "The sequence is not uniterrupted for files {} in directory {}",
            fg.get_file_name(),
            dir
        )))
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    for vcs in setup_dir.vcs_directories() {
        verify_uninterrupted_monotonic_sequence(
            vcs.setup_component_verification_data_payload_group(),
            result,
            &vcs.name(),
        );
        verify_uninterrupted_monotonic_sequence(
            vcs.control_component_code_shares_payload_group(),
            result,
            &vcs.name(),
        );
        for (i, elt) in vcs.setup_component_verification_data_payload_iter() {
            match elt {
                Ok(p) => {
                    if p.chunk_id != i {
                        result.push(VerificationEvent::new_failure(&format!(
                            "The chunkID nr {} does not matches the chunkID in the file name in {} for setup_component_verification_data_payload",
                            i,
                            vcs.name()
                        )))
                    }
                }
                Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                    "Error getting setup_component_verification_data_payload for chunk {} in {}",
                    i,
                    vcs.name()
                ))),
            }
        }
        for (i, elt) in vcs.control_component_code_shares_payload_iter() {
            match elt {
                Ok(p) => {
                    for (j, e) in p.0.iter().enumerate() {
                        if e.chunk_id != i {
                            result.push(VerificationEvent::new_failure(&format!(
                            "The chunkID nr {} does not matches the chunkID in the file name in {} for control_component_code_shares_payload at pos {}",
                            i,
                            vcs.name(), j
                        )))
                        }
                    }
                }
                Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                    "Error getting control_component_code_shares_payload for chunk {} in {}",
                    i,
                    vcs.name()
                ))),
            }
        }
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
