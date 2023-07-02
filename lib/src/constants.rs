//! Some constants for the program

use std::env::current_dir;
use std::path::{Path, PathBuf};

// Constants from specification
pub(crate) const MAXIMUM_NUMBER_OF_VOTING_OPTIONS: usize = 3000;
pub(crate) const MAXIMUM_NUMBER_OF_SELECTABLE_VOTING_OPTIONS: usize = 120;
//pub(crate) const MAXIMUM_NUMBER_OF_WRITE_IN_OPTIONS: usize = 15;
//pub(crate) const MAXIMUM_WRITE_IN_OPTION_LENGTH: usize = 500;
//pub(crate) const MAXIMUM_ACTUAL_VOTING_OPTION_LENGTH: usize = 50;

// Directory structure
pub const SETUP_DIR_NAME: &str = "setup";
pub const TALLY_DIR_NAME: &str = "tally";
pub(crate) const VCS_DIR_NAME: &str = "verification_card_sets";
pub(crate) const BB_DIR_NAME: &str = "ballot_boxes";

// Program structure
pub(crate) const LOG_PATH: &str = "log/log.txt";
pub(crate) const DIRECT_TRUST_PATH: &str = "direct_trust";
pub(crate) const RESOURCES_PATH: &str = "resources";
//pub(crate) const SCHEMA_PATH: &str = "schemas";
pub(crate) const VERIFICATION_LIST_PATH: &str = "verification_list.json";

fn root_dir(root_path: Option<&Path>) -> PathBuf {
    match root_path {
        Some(p) => p.to_path_buf(),
        None => {
            if cfg!(test) {
                current_dir().unwrap().parent().unwrap().to_path_buf()
            } else {
                current_dir().unwrap()
            }
        }
    }
}

pub fn verification_list_path(root_path: Option<&Path>) -> PathBuf {
    root_dir(root_path)
        .join(RESOURCES_PATH)
        .join(VERIFICATION_LIST_PATH)
}

pub fn direct_trust_path(root_path: Option<&Path>) -> PathBuf {
    root_dir(root_path).join(DIRECT_TRUST_PATH)
}

pub fn log_path(root_path: Option<&Path>) -> PathBuf {
    root_dir(root_path).join(LOG_PATH)
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::{file_structure::VerificationDirectory, verification::VerificationPeriod};
    use std::path::{Path, PathBuf};

    pub(crate) fn datasets_path() -> PathBuf {
        Path::new("..").join("datasets")
    }

    pub(crate) fn dataset_setup_path() -> PathBuf {
        datasets_path().join("dataset1-setup")
    }

    pub(crate) fn dataset_tally_path() -> PathBuf {
        datasets_path().join("dataset1-setup-tally")
    }

    pub(crate) fn get_verifier_tally_dir() -> VerificationDirectory {
        VerificationDirectory::new(&VerificationPeriod::Tally, &dataset_tally_path())
    }

    pub(crate) fn get_verifier_setup_dir() -> VerificationDirectory {
        VerificationDirectory::new(&VerificationPeriod::Setup, &dataset_setup_path())
    }

    #[test]
    fn test_check_verification_dir() {
        assert!(verification_list_path(None).exists())
    }

    #[test]
    fn test_log_path() {
        assert!(log_path(None).exists())
    }
}
