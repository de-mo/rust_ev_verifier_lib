use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    error::{create_verifier_error, VerifierError},
    file_structure::{
        file_group::FileGroup,
        setup_directory::{SetupDirectoryTrait, VCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use log::debug;

fn verify_uninterrupted_monotonic_sequence(
    fg: &FileGroup,
    result: &mut VerificationResult,
    dir: &String,
) {
    let mut numbers = fg.get_numbers().clone();
    numbers.sort();
    if !fg.has_elements() && !(numbers[0] + numbers[numbers.len() - 1] != numbers.len()) {
        result.push_failure(create_verification_failure!(format!(
            "The sequence is not uniterrupted for files {} in directory {}",
            fg.get_file_name(),
            dir
        )))
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    for vcs in setup_dir.vcs_directories() {
        verify_uninterrupted_monotonic_sequence(
            vcs.setup_component_verification_data_payload_group(),
            result,
            &vcs.get_name(),
        );
        verify_uninterrupted_monotonic_sequence(
            vcs.control_component_code_shares_payload_group(),
            result,
            &vcs.get_name(),
        );
        for (i, elt) in vcs.setup_component_verification_data_payload_iter() {
            match elt {
                Ok(p) => {
                    if p.chunk_id != i {
                        result.push_failure(create_verification_failure!(format!(
                            "The chunkID nr {} does not matches the chunkID in the file name in {} for setup_component_verification_data_payload",
                            i,
                            vcs.get_name()
                        )))
                    }
                }
                Err(e) => result.push_error(create_verification_error!(
                    format!(
                    "Error getting setup_component_verification_data_payload for chunk {} in {}",
                    i,
                    vcs.get_name()
                ),
                    e
                )),
            }
        }
        for (i, elt) in vcs.control_component_code_shares_payload_iter() {
            match elt {
                Ok(p) => {
                    for (j, e) in p.iter().enumerate() {
                        if e.chunk_id != i {
                            result.push_failure(create_verification_failure!(format!(
                            "The chunkID nr {} does not matches the chunkID in the file name in {} for control_component_code_shares_payload at pos {}",
                            i,
                            vcs.get_name(), j
                        )))
                        }
                    }
                }
                Err(e) => result.push_error(create_verification_error!(
                    format!(
                        "Error getting control_component_code_shares_payload for chunk {} in {}",
                        i,
                        vcs.get_name()
                    ),
                    e
                )),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::super::{verification::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::file_structure::VerificationDirectory;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset1-setup-tally");
        VerificationDirectory::new(&VerificationPeriod::Tally, &location)
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
