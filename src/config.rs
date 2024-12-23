//! Module containing the contstants and the way to access them

use super::consts;
use super::direct_trust::{DirectTrustError, Keystore};
use super::resources::VERIFICATION_LIST;
use std::path::{Path, PathBuf};

// Directory structure
const CONTEXT_DIR_NAME: &str = "context";
const SETUP_DIR_NAME: &str = "setup";
const TALLY_DIR_NAME: &str = "tally";
const VCS_DIR_NAME: &str = "verificationCardSets";
const BB_DIR_NAME: &str = "ballotBoxes";

// Program structure
const LOG_DIR_NAME: &str = "log";
const LOG_FILE_NAME: &str = "log.txt";
const DIRECT_TRUST_DIR_NAME: &str = "direct-trust";
const DATA_DIR_NAME: &str = "data";
const ZIP_TEMP_DIR_NAME: &str = "decrypted_zip";

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
    pub fn maximum_supported_number_of_write_in_options() -> usize {
        consts::MAXIMUM_SUPPORTED_NUMBER_OF_WRITE_IN_OPTIONS
    }

    /// Maximum supported number of write-in options + 1 to the specification
    pub fn delta_sup() -> usize {
        consts::MAXIMUM_SUPPORTED_NUMBER_OF_WRITE_IN_OPTIONS + 1
    }

    pub fn maximum_write_in_option_length() -> usize {
        consts::MAXIMUM_WRITE_IN_OPTION_LENGTH
    }

    /// Character length of unique identifiers
    pub fn l_id() -> usize {
        consts::CHARACTER_LENGTH_OF_UNIQUE_IDENTIFIERS
    }

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

    /// The name of the temp directory
    pub fn temp_dir_name() -> &'static str {
        DATA_DIR_NAME
    }

    /// The path to the log file
    pub fn log_file_path(&self) -> PathBuf {
        self.root_dir_path().join(LOG_DIR_NAME).join(LOG_FILE_NAME)
    }

    /// The path to the dir name
    /// Create the directory if not exist
    pub fn data_dir_path(&self) -> PathBuf {
        let res = self.root_dir_path().join(Self::temp_dir_name());
        if !res.is_dir() {
            let _ = std::fs::create_dir_all(&res);
        }
        res
    }

    /// The path to the dir name
    /// Create the directory if not exist
    pub fn zip_temp_dir_path(&self) -> PathBuf {
        let res = self.data_dir_path().join(ZIP_TEMP_DIR_NAME);
        if !res.is_dir() {
            let _ = std::fs::create_dir_all(&res);
        }
        res
    }

    /// Create a dataset directory and return the value.
    /// The dataset dir contains the current date time
    /// Create the directory if not exist
    pub fn create_dataset_dir_path(&self) -> PathBuf {
        let res = self.data_dir_path().join(format!(
            "dataset-{}",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        ));
        let _ = std::fs::create_dir_all(&res);
        res
    }

    /// The path to the directory where direct trust keystore is stored
    pub fn direct_trust_dir_path(&self) -> PathBuf {
        self.root_dir_path().join(DIRECT_TRUST_DIR_NAME)
    }

    /// Get the relative path of the file containing the configuration of the verifications
    pub fn get_verification_list_str(&self) -> &'static str {
        VERIFICATION_LIST
    }

    /// Get the keystore
    pub fn keystore(&self) -> Result<Keystore, DirectTrustError> {
        Keystore::try_from(self.direct_trust_dir_path().as_path())
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod test {
    use super::*;
    use crate::{
        direct_trust::CertificateAuthority,
        file_structure::{mock::MockVerificationDirectory, VerificationDirectory},
        verification::VerificationPeriod,
    };
    use lazy_static::lazy_static;
    use rust_ev_crypto_primitives::direct_trust::Keystore as BasisKeystore;

    const CANTON_KEYSTORE_FILE_NAME: &str = "signing_keystore_canton.p12";
    const CANTON_KEYSTORE_PASSWORD_FILE_NAME: &str = "signing_pw_canton.txt";
    const CC1_KEYSTORE_FILE_NAME: &str = "signing_keystore_control_component_1.p12";
    const CC1_KEYSTORE_PASSWORD_FILE_NAME: &str = "signing_pw_control_component_1.txt";
    const CC2_KEYSTORE_FILE_NAME: &str = "signing_keystore_control_component_2.p12";
    const CC2_KEYSTORE_PASSWORD_FILE_NAME: &str = "signing_pw_control_component_2.txt";
    const CC3_KEYSTORE_FILE_NAME: &str = "signing_keystore_control_component_3.p12";
    const CC3_KEYSTORE_PASSWORD_FILE_NAME: &str = "signing_pw_control_component_3.txt";
    const CC4_KEYSTORE_FILE_NAME: &str = "signing_keystore_control_component_3.p12";
    const CC4_KEYSTORE_PASSWORD_FILE_NAME: &str = "signing_pw_control_component_4.txt";
    const SETUP_KEYSTORE_FILE_NAME: &str = "signing_keystore_sdm_config.p12";
    const SETUP_KEYSTORE_PASSWORD_FILE_NAME: &str = "signing_pw_sdm_config.txt";
    const TALLY_KEYSTORE_FILE_NAME: &str = "signing_keystore_sdm_tally.p12";
    const TALLY_KEYSTORE_PASSWORD_FILE_NAME: &str = "signing_pw_sdm_tally.txt";

    const TEST_TEMP_DIR_NAME: &str = "test_temp_dir";
    const BB_ID_ONE_VOTE: &str = "A6733AB3D38BC47B964FB2DF09404877";
    const BB_ID_ZERO_VOTE: &str = "80DEAF0E396ACC1183EFD4E8A0BCD172";
    const BB_ID_MANY_VOTES: &str = "2705DAB8C622B8DD4EE6A0D490A914B4";
    const CONTEXT_ZIP_FILENAME: &str = "Dataset-context-NE_20231124_TT05-20241016_1513.zip";
    const SETUP_ZIP_FILENAME: &str = "Dataset-setup-NE_20231124_TT05-20241016_1513.zip";
    const TALLY_ZIP_FILENAME: &str = "Dataset-tally-NE_20231124_TT05-20241016_1707.zip";
    const TEST_DECRYPT_ZIP_PASSWORD: &str = "LongPassword_Encryption1";

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

    pub(crate) fn test_temp_dir_path() -> PathBuf {
        CONFIG_TEST.root_dir_path().join(TEST_TEMP_DIR_NAME)
    }

    pub(crate) fn test_datasets_context_zip_path() -> PathBuf {
        test_datasets_path().join(CONTEXT_ZIP_FILENAME)
    }

    pub(crate) fn test_datasets_setup_zip_path() -> PathBuf {
        test_datasets_path().join(SETUP_ZIP_FILENAME)
    }

    pub(crate) fn test_datasets_tally_zip_path() -> PathBuf {
        test_datasets_path().join(TALLY_ZIP_FILENAME)
    }

    pub(crate) fn test_decrypt_zip_password() -> &'static str {
        TEST_DECRYPT_ZIP_PASSWORD
    }

    pub(crate) fn test_all_paths_of_subdir(
        fn_path: &dyn Fn() -> PathBuf,
        subdir: &str,
    ) -> Vec<PathBuf> {
        let test_bbs_path = fn_path().join(subdir);
        std::fs::read_dir(test_bbs_path.as_path())
            .unwrap()
            .filter_map(|res| res.ok())
            .map(|f| f.path())
            .collect()
    }

    pub(crate) fn test_all_context_vcs_paths() -> Vec<PathBuf> {
        test_all_paths_of_subdir(&test_datasets_context_path, "verificationCardSets")
    }

    pub(crate) fn test_all_setup_vcs_paths() -> Vec<PathBuf> {
        test_all_paths_of_subdir(&test_datasets_setup_path, "verificationCardSets")
    }

    pub(crate) fn test_ballot_box_one_vote_path() -> PathBuf {
        test_datasets_tally_path()
            .join("ballotBoxes")
            .join(BB_ID_ONE_VOTE)
    }

    pub(crate) fn test_ballot_box_zero_vote_path() -> PathBuf {
        test_datasets_tally_path()
            .join("ballotBoxes")
            .join(BB_ID_ZERO_VOTE)
    }

    pub(crate) fn test_ballot_box_many_votes_path() -> PathBuf {
        test_datasets_tally_path()
            .join("ballotBoxes")
            .join(BB_ID_MANY_VOTES)
    }

    pub(crate) fn test_context_verification_card_set_path() -> PathBuf {
        test_all_context_vcs_paths()[0].clone()
    }

    pub(crate) fn test_setup_verification_card_set_path() -> PathBuf {
        test_all_setup_vcs_paths()[0].clone()
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

    pub(crate) fn get_test_verifier_mock_setup_dir() -> MockVerificationDirectory {
        MockVerificationDirectory::new(&VerificationPeriod::Setup, &test_datasets_path())
    }

    #[allow(dead_code)]
    pub(crate) fn get_test_verifier_mock_tally_dir() -> MockVerificationDirectory {
        MockVerificationDirectory::new(&VerificationPeriod::Setup, &test_datasets_path())
    }

    pub(crate) fn get_test_signing_direct_trust_path() -> PathBuf {
        test_resources_path().join("signing_keystore")
    }

    /// Get the signing keystore
    pub fn signing_keystore(
        authority: CertificateAuthority,
    ) -> Result<BasisKeystore, DirectTrustError> {
        let (ks_name, pwd_name) = match authority {
            CertificateAuthority::Canton => (
                CANTON_KEYSTORE_FILE_NAME,
                CANTON_KEYSTORE_PASSWORD_FILE_NAME,
            ),
            CertificateAuthority::SdmConfig => {
                (SETUP_KEYSTORE_FILE_NAME, SETUP_KEYSTORE_PASSWORD_FILE_NAME)
            }
            CertificateAuthority::SdmTally => {
                (TALLY_KEYSTORE_FILE_NAME, TALLY_KEYSTORE_PASSWORD_FILE_NAME)
            }
            CertificateAuthority::ControlComponent1 => {
                (CC1_KEYSTORE_FILE_NAME, CC1_KEYSTORE_PASSWORD_FILE_NAME)
            }
            CertificateAuthority::ControlComponent2 => {
                (CC2_KEYSTORE_FILE_NAME, CC2_KEYSTORE_PASSWORD_FILE_NAME)
            }
            CertificateAuthority::ControlComponent3 => {
                (CC3_KEYSTORE_FILE_NAME, CC3_KEYSTORE_PASSWORD_FILE_NAME)
            }
            CertificateAuthority::ControlComponent4 => {
                (CC4_KEYSTORE_FILE_NAME, CC4_KEYSTORE_PASSWORD_FILE_NAME)
            }
        };
        BasisKeystore::from_pkcs12(
            &get_test_signing_direct_trust_path().join(ks_name),
            &get_test_signing_direct_trust_path().join(pwd_name),
        )
        .map_err(|e| DirectTrustError::Keystore {
            msg: "Problem reading the keystore for test signing".to_string(),
            source: e,
        })
    }

    pub(crate) fn test_resources_path() -> PathBuf {
        CONFIG_TEST.root_dir_path().join("resources").join("test")
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
