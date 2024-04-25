//! Module containing the contstants and the way to access them

use super::consts;
use super::resources::VERIFICATION_LIST;
use anyhow::{Context, Result};
use rust_ev_crypto_primitives::Keystore;
use std::path::{Path, PathBuf};

// Directory structure
pub const CONTEXT_DIR_NAME: &str = "context";
pub const SETUP_DIR_NAME: &str = "setup";
pub const TALLY_DIR_NAME: &str = "tally";
const VCS_DIR_NAME: &str = "verificationCardSets";
const BB_DIR_NAME: &str = "ballotBoxes";

// Program structure
const LOG_DIR_NAME: &str = "log";
const LOG_FILE_NAME: &str = "log.txt";
const DIRECT_TRUST_DIR_NAME: &str = "direct-trust";
const KEYSTORE_FILE_NAME: &str = "public_keys_keystore_verifier.p12";
const KEYSTORE_PASSWORD_FILE_NAME: &str = "public_keys_keystore_verifier_pw.txt";

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

    /// Path of the root directory of the programm
    pub fn root_dir_path(&self) -> PathBuf {
        Path::new(self.0).to_path_buf()
    }

    /// Maximum number of voting options according to the specification
    pub fn maximum_number_of_supported_voting_options_n_sup() -> usize {
        consts::MAXIMUM_NUMBER_OF_SUPPORTED_VOTING_OPTIONS_N_SUP
    }

    /// Maximum number of of selectable voting options according to the specification
    pub fn maximum_supported_number_of_selections_psi_sup() -> usize {
        consts::MAXIMUM_SUPPORTED_NUMBER_OF_SELECTIONS_PSI_SUP
    }

    /// Maximum supported number of write-in options according to the specification
    #[allow(dead_code)]
    pub fn maximum_supported_number_of_write_in_options() -> usize {
        consts::MAXIMUM_SUPPORTED_NUMBER_OF_WRITE_IN_OPTIONS
    }

    /// Maximum supported number of write-in options + 1 to the specification
    #[allow(dead_code)]
    pub fn delta_sup() -> usize {
        consts::MAXIMUM_SUPPORTED_NUMBER_OF_WRITE_IN_OPTIONS + 1
    }

    #[allow(dead_code)]
    pub fn maximum_write_in_option_length() -> usize {
        consts::MAXIMUM_WRITE_IN_OPTION_LENGTH
    }

    /// Character length of unique identifiers
    pub fn l_id() -> usize {
        consts::CHARACTER_LENGTH_OF_UNIQUE_IDENTIFIERS
    }

    #[allow(dead_code)]
    pub fn maximum_actual_voting_option_length() -> usize {
        consts::MAXIMUM_ACTUAL_VOTING_OPTION_LENGTH
    }

    /// The name of the context directory
    pub fn context_dir_name() -> &'static str {
        CONTEXT_DIR_NAME
    }

    /// The name of the setup directory
    pub fn setup_dir_name() -> &'static str {
        SETUP_DIR_NAME
    }

    /// The name of the tally directory
    pub fn tally_dir_name() -> &'static str {
        TALLY_DIR_NAME
    }

    /// The name of the vcs (voting card sets) directories
    pub fn vcs_dir_name() -> &'static str {
        VCS_DIR_NAME
    }

    /// The name of the bb (ballot boxes) directories
    pub fn bb_dir_name() -> &'static str {
        BB_DIR_NAME
    }

    /// The path to the log file
    pub fn log_file_path(&self) -> PathBuf {
        self.root_dir_path().join(LOG_DIR_NAME).join(LOG_FILE_NAME)
    }

    /// The path to the directory where direct trust keystore is stored
    fn direct_trust_dir_path(&self) -> PathBuf {
        self.root_dir_path().join(DIRECT_TRUST_DIR_NAME)
    }

    pub fn direct_trust_keystore_path(&self) -> PathBuf {
        self.direct_trust_dir_path().join(KEYSTORE_FILE_NAME)
    }

    pub fn direct_trust_keystore_password_path(&self) -> PathBuf {
        self.direct_trust_dir_path()
            .join(KEYSTORE_PASSWORD_FILE_NAME)
    }

    /// Get the relative path of the file containing the configuration of the verifications
    pub fn get_verification_list_str(&self) -> &'static str {
        VERIFICATION_LIST
    }

    /// Get the keystore
    pub fn keystore(&self) -> Result<Keystore> {
        Keystore::from_pkcs12(
            &self.direct_trust_keystore_path(),
            &self.direct_trust_keystore_password_path(),
        )
        .context("Problem reading the keystore")
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

    pub(crate) fn test_datasets_tally_path() -> PathBuf {
        test_datasets_path().join(TALLY_DIR_NAME)
    }

    pub(crate) fn test_datasets_setup_path() -> PathBuf {
        test_datasets_path().join(SETUP_DIR_NAME)
    }

    pub(crate) fn test_datasets_context_path() -> PathBuf {
        test_datasets_path().join(CONTEXT_DIR_NAME)
    }

    pub(crate) fn test_ballot_box_path() -> PathBuf {
        test_datasets_tally_path()
            .join("ballotBoxes")
            .join("915C2B2C9D6631BABDB6069D6CD6496D")
    }

    pub(crate) fn test_ballot_box_empty_path() -> PathBuf {
        test_datasets_tally_path()
            .join("ballotBoxes")
            .join("D9FA8F23294682554F536AB040AC2294")
    }

    pub(crate) fn test_context_verification_card_set_path() -> PathBuf {
        test_datasets_context_path()
            .join("verificationCardSets")
            .join("41AEF809A62BF53C7CFF61AA0925F636")
    }

    pub(crate) fn test_setup_verification_card_set_path() -> PathBuf {
        test_datasets_setup_path()
            .join("verificationCardSets")
            .join("41AEF809A62BF53C7CFF61AA0925F636")
    }

    pub(crate) fn test_xml_path() -> PathBuf {
        CONFIG_TEST.root_dir_path().join("xml_tests")
    }

    pub(crate) fn get_test_verifier_tally_dir() -> VerificationDirectory {
        VerificationDirectory::new(&VerificationPeriod::Tally, &test_datasets_path())
    }

    pub(crate) fn get_test_verifier_setup_dir() -> VerificationDirectory {
        VerificationDirectory::new(&VerificationPeriod::Setup, &test_datasets_path())
    }

    #[test]
    fn test_config() {
        let c = Config::default();
        assert_eq!(c.root_dir_path(), Path::new("."));
        assert_eq!(c.log_file_path(), Path::new("./log/log.txt"));
        assert_eq!(c.direct_trust_dir_path(), Path::new("./direct-trust"));
        assert!(!c.get_verification_list_str().is_empty());
    }
}
