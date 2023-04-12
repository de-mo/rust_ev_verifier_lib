//! Some constants for the program

use std::path::{Path, PathBuf};

// Constants from specification
pub const MAXIMUM_NUMBER_OF_VOTING_OPTIONS: usize = 3000;
pub const MAXIMUM_NUMBER_OF_SELECTABLE_VOTING_OPTIONS: usize = 120;
pub const MAXIMUM_NUMBER_OF_WRITE_IN_OPTIONS: usize = 15;
pub const MAXIMUM_WRITE_IN_OPTION_LENGTH: usize = 500;
pub const MAXIMUM_ACTUAL_VOTING_OPTION_LENGTH: usize = 50;

// Directory structure
pub const SETUP_DIR_NAME: &str = "setup";
pub const TALLY_DIR_NAME: &str = "tally";
pub const VCS_DIR_NAME: &str = "verification_card_sets";
pub const BB_DIR_NAME: &str = "ballot_boxes";

// Program structure
pub const LOG_PATH: &str = "log/log.txt";
pub const DIRECT_TRUST_PATH: &str = "direct_trust";
pub const RESOURCES_PATH: &str = "resources";
pub const SCHEMA_PATH: &str = "schemas";
pub const VERIFICATION_LIST_PATH: &str = "verification_list.json";

//
pub fn verification_list_path() -> PathBuf {
    Path::new(".")
        .join(RESOURCES_PATH)
        .join(VERIFICATION_LIST_PATH)
}
