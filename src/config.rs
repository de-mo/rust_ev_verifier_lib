use std::path::{Path, PathBuf};

// Constants from specification
const MAXIMUM_NUMBER_OF_VOTING_OPTIONS: usize = 3000;
const MAXIMUM_NUMBER_OF_SELECTABLE_VOTING_OPTIONS: usize = 120;
const MAXIMUM_NUMBER_OF_WRITE_IN_OPTIONS: usize = 15;
const MAXIMUM_WRITE_IN_OPTION_LENGTH: usize = 500;
const MAXIMUM_ACTUAL_VOTING_OPTION_LENGTH: usize = 50;

// Directory structure
pub const SETUP_DIR_NAME: &str = "setup";
pub const TALLY_DIR_NAME: &str = "tally";
const VCS_DIR_NAME: &str = "verification_card_sets";
const BB_DIR_NAME: &str = "ballot_boxes";

// Program structure
const LOG_DIR_NAME: &str = "log";
const LOG_FILE_NAME: &str = "log.txt";
const DIRECT_TRUST_DIR_NAME: &str = "direct_trust";
//const RESOURCES_DIR_NAME: &str = "resources";
// const SCHEMA_PATH: &str = "schemas";
//const VERIFICATION_LIST_FILE_NAME: &str = "verification_list.json";

static VERIFICATION_LIST: &str = include_str!("../resources/verification_list.json");

/// Structuring getting all the configuration information relevant for the
/// verifier
///
/// The structure get only the root directory of the running application. The structure
/// can be defined as static using lazy_static crate:
/// ```ignore
/// use lazy_static::lazy_static;
/// lazy_static! {
///     static ref CONFIG: Config = Config::new("..");
///  }
/// ```
pub struct Config(&'static str);

/// New config with root_dir equal "."
impl Default for Config {
    fn default() -> Self {
        Self::new(".")
    }
}

impl Config {
    /// New Config
    pub fn new(root_dir: &'static str) -> Self {
        Config(root_dir)
    }

    pub fn root_dir_path(&self) -> PathBuf {
        Path::new(self.0).to_path_buf()
    }

    pub fn maximum_number_of_voting_options() -> usize {
        MAXIMUM_NUMBER_OF_VOTING_OPTIONS
    }

    pub fn maximum_number_of_selectable_voting_options() -> usize {
        MAXIMUM_NUMBER_OF_SELECTABLE_VOTING_OPTIONS
    }

    pub fn maximum_number_of_write_in_options() -> usize {
        MAXIMUM_NUMBER_OF_WRITE_IN_OPTIONS
    }

    pub fn maximum_write_in_option_length() -> usize {
        MAXIMUM_WRITE_IN_OPTION_LENGTH
    }

    pub fn maximum_actual_voting_option_length() -> usize {
        MAXIMUM_ACTUAL_VOTING_OPTION_LENGTH
    }

    pub fn setup_dir_name() -> &'static str {
        SETUP_DIR_NAME
    }

    pub fn tally_dir_name() -> &'static str {
        TALLY_DIR_NAME
    }

    pub fn vcs_dir_name() -> &'static str {
        VCS_DIR_NAME
    }

    pub fn bb_dir_name() -> &'static str {
        BB_DIR_NAME
    }

    pub fn log_file_path(&self) -> PathBuf {
        self.root_dir_path().join(LOG_DIR_NAME).join(LOG_FILE_NAME)
    }

    pub fn direct_trust_dir_path(&self) -> PathBuf {
        self.root_dir_path().join(DIRECT_TRUST_DIR_NAME)
    }

    pub fn get_verification_list_str(&self) -> &'static str {
        VERIFICATION_LIST
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::{file_structure::VerificationDirectory, verification::VerificationPeriod};
    use lazy_static::lazy_static;

    lazy_static! {
        pub(crate) static ref CONFIG_TEST: Config = Config::new(".");
    }

    pub(crate) fn test_datasets_path() -> PathBuf {
        CONFIG_TEST.root_dir_path().join("datasets")
    }

    pub(crate) fn test_dataset_setup_path() -> PathBuf {
        test_datasets_path().join("dataset1-setup")
    }

    pub(crate) fn test_dataset_tally_path() -> PathBuf {
        test_datasets_path().join("dataset1-setup-tally")
    }

    pub(crate) fn get_test_verifier_tally_dir() -> VerificationDirectory {
        VerificationDirectory::new(&VerificationPeriod::Tally, &test_dataset_tally_path())
    }

    pub(crate) fn get_test_verifier_setup_dir() -> VerificationDirectory {
        VerificationDirectory::new(&VerificationPeriod::Setup, &test_dataset_setup_path())
    }

    #[test]
    fn test_config() {
        let c = Config::default();
        assert_eq!(c.root_dir_path(), Path::new("."));
        assert_eq!(c.log_file_path(), Path::new("./log/log.txt"));
        assert_eq!(c.direct_trust_dir_path(), Path::new("./direct_trust"));
        assert!(!c.get_verification_list_str().is_empty());
    }
}
