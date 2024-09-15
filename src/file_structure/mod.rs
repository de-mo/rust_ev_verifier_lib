//! Module implementing the structure of files and directories
//! to collect data for the verifications
//!
pub mod context_directory;
pub mod file;
pub mod file_group;
pub mod setup_directory;
pub mod tally_directory;

use self::{
    context_directory::{ContextDirectory, ContextDirectoryTrait},
    setup_directory::SetupDirectoryTrait,
    tally_directory::TallyDirectoryTrait,
};
use crate::{
    data_structures::{
        context::VerifierContextDataType, setup::VerifierSetupDataType,
        tally::VerifierTallyDataType, DataStructureError, VerifierDataType,
    },
    verification::VerificationPeriod,
};
use roxmltree::Error as RoXmlTreeError;
use setup_directory::SetupDirectory;
use std::path::{Path, PathBuf};
use tally_directory::TallyDirectory;
use thiserror::Error;

// Enum representing the direct trust errors
#[derive(Error, Debug)]
pub enum FileStructureError {
    #[error("IO error for {path} -> caused by: {source}")]
    IO {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Path is not a file {0}")]
    PathNotFile(PathBuf),
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    ParseRoXML { msg: String, source: RoXmlTreeError },
    #[error("Mock error: {0}")]
    Mock(String),
    #[error("Error reading data structure {path} -> caudes by: {source}")]
    ReadDataStructure {
        path: PathBuf,
        source: DataStructureError,
    },
    #[error("Path is not a directory {0}")]
    PathIsNotDir(PathBuf),
    /*
    #[error("Path doesn't exists {0}")]
    PathNotExist(PathBuf),
    #[error("Path is not a directory {0}")]
    PathIsNotDir(PathBuf),
    #[error("Crypto error {msg} -> caused by: {source}")]
    CryptoError {
        msg: String,
        source: BasisCryptoError,
    },
    #[error("Byte length error {0}")]
    ByteLengthError(String),
    #[error("Error Unzipping {file}: {source}")]
    Unzip {
        file: PathBuf,
        source: zip_extract::ZipExtractError,
    },
    #[error("Kind {0} delivered. Only context, setup and tally possible")]
    WrongKindStr(String), */
}

#[derive(Clone)]
/// Type represending a VerificationDirectory (subdirectory context and setup or tally)
pub struct VerificationDirectory {
    context: ContextDirectory,
    setup: Option<SetupDirectory>,
    tally: Option<TallyDirectory>,
}

/// Enum to define the type of the file (Json or Xml)
pub enum FileType {
    Json,
    Xml,
}

/// Enum representing the mode to read a fie (Memory or streaming).
pub enum FileReadMode {
    Memory,
    Streaming,
}

/// Trait defining functions to get the filename
pub trait GetFileNameTrait {
    /// Get the file name as it is defiened
    fn get_raw_file_name(&self) -> String;

    /// Get the filename injecting the number (if given)
    ///
    /// # Argument
    /// * `value`: The value as option
    ///
    /// # return
    /// The name replacing `{}` with the given value (if is some).
    ///
    /// # Example
    /// ```rust
    /// use rust_verifier::file_structure::GetFileNameTrait;
    /// struct Test;
    /// impl GetFileNameTrait for Test {
    ///     fn get_raw_file_name(&self) -> String {
    ///         String::from("new_{}")
    ///     }
    /// };
    /// let t = Test {};
    /// assert_eq!(t.get_file_name(None), "new_{}");
    /// assert_eq!(t.get_file_name(Some(2)), "new_2");
    /// ```
    fn get_file_name(&self, value: Option<usize>) -> String {
        let s = self.get_raw_file_name();
        match value {
            Some(i) => s.replace("{}", &i.to_string()),
            None => s,
        }
    }
}

/// Trait to set the necessary functions for the struct [VerificationDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
///
/// The verification functions should only defined with the traits as follow
/// ```ignore
/// fn fn_verification<'a, D: VerificationDirectoryTrait>(
///    dir: &D,
///    result: &mut VerificationResult,
/// ) {
///     ...
/// }
/// ```
///
/// All the helpers functions called from `fn_verification` have also to take then traits as parameter
/// and not the structs. Then it is possible to mock the data
pub trait VerificationDirectoryTrait {
    type ContextDirType: ContextDirectoryTrait;
    type SetupDirType: SetupDirectoryTrait;
    type TallyDirType: TallyDirectoryTrait;

    /// Reference to context
    fn context(&self) -> &Self::ContextDirType;

    /// Unwrap setup and give a reference to the directory
    ///
    /// panic if other type
    fn unwrap_setup(&self) -> &Self::SetupDirType;

    /// Unwrap tally and give a reference to the directory
    ///
    /// panic if other type
    fn unwrap_tally(&self) -> &Self::TallyDirType;
}

pub trait CompletnessTestTrait {
    fn test_completness(&self) -> Result<Vec<String>, FileStructureError>;
}

impl VerificationDirectory {
    /// Create a new VerificationDirectory
    pub fn new(period: &VerificationPeriod, location: &Path) -> Self {
        let context = ContextDirectory::new(location);
        match period {
            VerificationPeriod::Setup => VerificationDirectory {
                context,
                setup: Some(SetupDirectory::new(location)),
                tally: None,
            },
            VerificationPeriod::Tally => VerificationDirectory {
                context,
                setup: None,
                tally: Some(TallyDirectory::new(location)),
            },
        }
    }

    /// Is setup
    #[allow(dead_code)]
    pub fn is_setup(&self) -> bool {
        self.setup.is_some()
    }

    /// Is tally
    #[allow(dead_code)]
    pub fn is_tally(&self) -> bool {
        self.tally.is_some()
    }

    /// Are the entries valid
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.is_setup() != self.is_tally()
    }
}

impl VerificationDirectoryTrait for VerificationDirectory {
    type ContextDirType = ContextDirectory;
    type SetupDirType = SetupDirectory;
    type TallyDirType = TallyDirectory;

    fn context(&self) -> &ContextDirectory {
        &self.context
    }

    /// Unwrap setup and give a reference to S
    ///
    /// panic if type is tally
    fn unwrap_setup(&self) -> &SetupDirectory {
        match &self.setup {
            Some(s) => s,
            None => panic!("called `unwrap_setup()` on a `Tally` value"),
        }
    }

    /// Unwrap tally and give a reference to S
    ///
    /// panic if type is seup
    fn unwrap_tally(&self) -> &TallyDirectory {
        match &self.tally {
            Some(t) => t,
            None => panic!("called `unwrap_tally()` on a `Setup` value"),
        }
    }
}

impl GetFileNameTrait for VerifierContextDataType {
    fn get_raw_file_name(&self) -> String {
        let s = match self {
            Self::ElectionEventContextPayload => "electionEventContextPayload.json",
            Self::SetupComponentPublicKeysPayload => "setupComponentPublicKeysPayload.json",
            Self::ControlComponentPublicKeysPayload => "controlComponentPublicKeysPayload.{}.json",
            Self::SetupComponentTallyDataPayload => "setupComponentTallyDataPayload.json",
            Self::ElectionEventConfiguration => "configuration-anonymized.xml",
        };
        s.to_string()
    }
}

impl From<VerifierContextDataType> for FileReadMode {
    fn from(value: VerifierContextDataType) -> Self {
        match value {
            VerifierContextDataType::ElectionEventContextPayload => FileReadMode::Memory,
            VerifierContextDataType::SetupComponentPublicKeysPayload => FileReadMode::Memory,
            VerifierContextDataType::ControlComponentPublicKeysPayload => FileReadMode::Memory,
            VerifierContextDataType::SetupComponentTallyDataPayload => FileReadMode::Memory,
            VerifierContextDataType::ElectionEventConfiguration => FileReadMode::Streaming,
        }
    }
}

impl From<VerifierContextDataType> for FileType {
    fn from(value: VerifierContextDataType) -> Self {
        match value {
            VerifierContextDataType::ElectionEventContextPayload => FileType::Json,
            VerifierContextDataType::SetupComponentPublicKeysPayload => FileType::Json,
            VerifierContextDataType::ControlComponentPublicKeysPayload => FileType::Json,
            VerifierContextDataType::SetupComponentTallyDataPayload => FileType::Json,
            VerifierContextDataType::ElectionEventConfiguration => FileType::Xml,
        }
    }
}

impl GetFileNameTrait for VerifierSetupDataType {
    fn get_raw_file_name(&self) -> String {
        let s = match self {
            Self::SetupComponentVerificationDataPayload => {
                "setupComponentVerificationDataPayload.{}.json"
            }
            Self::ControlComponentCodeSharesPayload => "controlComponentCodeSharesPayload.{}.json",
        };
        s.to_string()
    }
}

impl From<VerifierSetupDataType> for FileReadMode {
    fn from(value: VerifierSetupDataType) -> Self {
        match value {
            VerifierSetupDataType::SetupComponentVerificationDataPayload => FileReadMode::Memory,
            VerifierSetupDataType::ControlComponentCodeSharesPayload => FileReadMode::Memory,
        }
    }
}

impl From<VerifierSetupDataType> for FileType {
    fn from(value: VerifierSetupDataType) -> Self {
        match value {
            VerifierSetupDataType::SetupComponentVerificationDataPayload => FileType::Json,
            VerifierSetupDataType::ControlComponentCodeSharesPayload => FileType::Json,
        }
    }
}

impl GetFileNameTrait for VerifierTallyDataType {
    fn get_raw_file_name(&self) -> String {
        let s = match self {
            Self::ECH0110 => "eCH-0110_*.xml",
            Self::EVotingDecrypt => "evoting-decrypt_*.xml",
            Self::ECH0222 => "eCH-0222_*.xml",
            Self::TallyComponentVotesPayload => "tallyComponentVotesPayload.json",
            Self::TallyComponentShufflePayload => "tallyComponentShufflePayload.json",
            Self::ControlComponentBallotBoxPayload => "controlComponentBallotBoxPayload_{}.json",
            Self::ControlComponentShufflePayload => "controlComponentShufflePayload_{}.json",
        };
        s.to_string()
    }
}

impl From<VerifierTallyDataType> for FileReadMode {
    fn from(value: VerifierTallyDataType) -> Self {
        match value {
            VerifierTallyDataType::EVotingDecrypt => FileReadMode::Memory,
            VerifierTallyDataType::ECH0110 => FileReadMode::Memory,
            VerifierTallyDataType::ECH0222 => FileReadMode::Memory,
            VerifierTallyDataType::TallyComponentVotesPayload => FileReadMode::Memory,
            VerifierTallyDataType::TallyComponentShufflePayload => FileReadMode::Memory,
            VerifierTallyDataType::ControlComponentBallotBoxPayload => FileReadMode::Memory,
            VerifierTallyDataType::ControlComponentShufflePayload => FileReadMode::Memory,
        }
    }
}

impl From<VerifierTallyDataType> for FileType {
    fn from(value: VerifierTallyDataType) -> Self {
        match value {
            VerifierTallyDataType::EVotingDecrypt => FileType::Xml,
            VerifierTallyDataType::ECH0110 => FileType::Xml,
            VerifierTallyDataType::ECH0222 => FileType::Xml,
            VerifierTallyDataType::TallyComponentVotesPayload => FileType::Json,
            VerifierTallyDataType::TallyComponentShufflePayload => FileType::Json,
            VerifierTallyDataType::ControlComponentBallotBoxPayload => FileType::Json,
            VerifierTallyDataType::ControlComponentShufflePayload => FileType::Json,
        }
    }
}

impl GetFileNameTrait for VerifierDataType {
    fn get_raw_file_name(&self) -> String {
        match self {
            VerifierDataType::Context(t) => t.get_raw_file_name(),
            VerifierDataType::Setup(t) => t.get_raw_file_name(),
            VerifierDataType::Tally(t) => t.get_raw_file_name(),
        }
    }
}

impl From<VerifierDataType> for FileReadMode {
    fn from(value: VerifierDataType) -> Self {
        match value {
            VerifierDataType::Context(t) => FileReadMode::from(t),
            VerifierDataType::Setup(t) => FileReadMode::from(t),
            VerifierDataType::Tally(t) => FileReadMode::from(t),
        }
    }
}

impl From<VerifierDataType> for FileType {
    fn from(value: VerifierDataType) -> Self {
        match value {
            VerifierDataType::Context(t) => FileType::from(t),
            VerifierDataType::Setup(t) => FileType::from(t),
            VerifierDataType::Tally(t) => FileType::from(t),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        test_ballot_box_path, test_context_verification_card_set_path, test_datasets_context_path,
        test_setup_verification_card_set_path,
    };

    #[test]
    fn test_context_files_exist() {
        let path = test_datasets_context_path();
        assert!(path
            .join(
                VerifierDataType::Context(VerifierContextDataType::ElectionEventContextPayload)
                    .get_file_name(None)
            )
            .exists());
        assert!(path
            .join(
                VerifierDataType::Context(VerifierContextDataType::SetupComponentPublicKeysPayload)
                    .get_file_name(None)
            )
            .exists());
        let path2 = test_context_verification_card_set_path();
        assert!(path2
            .join(
                VerifierDataType::Context(VerifierContextDataType::SetupComponentTallyDataPayload)
                    .get_file_name(None)
            )
            .exists());
    }

    #[test]
    fn test_tally_files_exist() {
        let path2 = test_ballot_box_path();
        assert!(path2
            .join(
                VerifierDataType::Tally(VerifierTallyDataType::TallyComponentVotesPayload)
                    .get_file_name(None)
            )
            .exists());
        println!(
            "{:?}",
            VerifierDataType::Tally(VerifierTallyDataType::TallyComponentShufflePayload)
                .get_file_name(None)
        );
        println!(
            "{:?}",
            path2.join(
                VerifierDataType::Tally(VerifierTallyDataType::TallyComponentShufflePayload)
                    .get_file_name(None)
            )
        );
        assert!(path2
            .join(
                VerifierDataType::Tally(VerifierTallyDataType::TallyComponentShufflePayload)
                    .get_file_name(None)
            )
            .exists());
    }

    #[test]
    fn test_context_groups_exist() {
        let path = test_datasets_context_path();
        assert!(path
            .join(
                VerifierDataType::Context(
                    VerifierContextDataType::ControlComponentPublicKeysPayload
                )
                .get_file_name(Some(1))
            )
            .exists());
    }

    #[test]
    fn test_setup_groups_exist() {
        let path2 = test_setup_verification_card_set_path();
        println!("{:?}", path2);
        assert!(path2
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::ControlComponentCodeSharesPayload)
                    .get_file_name(Some(1))
            )
            .exists());
        assert!(path2
            .join(
                VerifierDataType::Setup(
                    VerifierSetupDataType::SetupComponentVerificationDataPayload
                )
                .get_file_name(Some(1))
            )
            .exists());
    }
}

#[cfg(any(test, doc))]
#[allow(dead_code)]
pub mod mock {
    //! Module defining mocking structure for [VerificationDirectory]
    //!
    //! Example of usage:
    //! ```ignore
    //!    let mut mock_dir = MockVerificationDirectory::new(&VerificationPeriod::Setup, &location);
    //!    // Collect the correct data
    //!    let mut eec = mock_dir
    //!        .unwrap_setup()
    //!        .election_event_context_payload()
    //!        .unwrap();
    //!    // Change the data
    //!    eec.encryption_group.p = BigUint::from(1234usize);
    //!    // Mock the dir with the faked data
    //!    mock_dir
    //!        .unwrap_setup_mut()
    //!        .mock_election_event_context_payload(&Ok(&eec));
    //!    // Test the verification that should generate failures
    //!    fn_verification(&mock_dir, &mut result);
    //! ```
    use super::{
        context_directory::mock::MockContextDirectory, setup_directory::mock::MockSetupDirectory,
        tally_directory::mock::MockTallyDirectory, *,
    };

    /// Mock for [VerificationDirectory]
    pub struct MockVerificationDirectory {
        context: MockContextDirectory,
        setup: Option<MockSetupDirectory>,
        tally: Option<MockTallyDirectory>,
    }

    impl VerificationDirectoryTrait for MockVerificationDirectory {
        type ContextDirType = MockContextDirectory;
        type SetupDirType = MockSetupDirectory;
        type TallyDirType = MockTallyDirectory;

        fn unwrap_setup(&self) -> &MockSetupDirectory {
            match &self.setup {
                Some(t) => t,
                None => panic!("called `unwrap_setup()` on a `Tally` value"),
            }
        }

        fn unwrap_tally(&self) -> &MockTallyDirectory {
            match &self.tally {
                Some(t) => t,
                None => panic!("called `unwrap_tally()` on a `Setup` value"),
            }
        }

        fn context(&self) -> &Self::ContextDirType {
            &self.context
        }
    }

    impl MockVerificationDirectory {
        /// Create a new [MockVerificationDirectory]
        pub fn new(period: &VerificationPeriod, location: &Path) -> Self {
            let context = MockContextDirectory::new(location);
            match period {
                VerificationPeriod::Setup => MockVerificationDirectory {
                    context,
                    setup: Some(MockSetupDirectory::new(location)),
                    tally: None,
                },
                VerificationPeriod::Tally => MockVerificationDirectory {
                    context,
                    setup: None,
                    tally: Some(MockTallyDirectory::new(location)),
                },
            }
        }

        /// Context mut
        pub fn context_mut(&mut self) -> &mut MockContextDirectory {
            &mut self.context
        }

        /// Unwrap [MockSetupDirectory] as mutable
        pub fn unwrap_setup_mut(&mut self) -> &mut MockSetupDirectory {
            match &mut self.setup {
                Some(t) => t,
                None => panic!("called `unwrap_tally()` on a `Setup` value"),
            }
        }

        /// Unwrap [TallyDirectory] as mutable
        pub fn unwrap_tally_mut(&mut self) -> &mut MockTallyDirectory {
            match &mut self.tally {
                Some(t) => t,
                None => panic!("called `unwrap_tally()` on a `Setup` value"),
            }
        }
    }

    /// Macro to implement a function to the mocked structures wrapping
    /// the function of the existing object for getter of File or FileGroup.
    /// If the value is mocked, then return the mocked value, else return the original.
    ///
    /// Parameters:
    /// - $fct: The name of the function
    /// - $mock: The name of the mocked structure field to get
    /// - $out: Type of the output of the function
    macro_rules! wrap_file_group_getter {
        ($fct: ident, $mock: ident, $out: ty) => {
            fn $fct(&self) -> &$out {
                match &self.$mock {
                    Some(e) => e,
                    None => self.dir.$fct(),
                }
            }
        };
    }
    pub(super) use wrap_file_group_getter;

    /// Macro to implement a function to the mocked structures wrapping
    /// the function of the existing object for getter of payload.
    /// If the value is mocked, then return the mocked value, else return the original.
    ///
    /// Parameters:
    /// - $fct: The name of the function
    /// - $mock: The name of the mocked structure field to get
    /// - $payload: Type of the pyalod
    macro_rules! wrap_payload_getter {
        ($fct: ident, $mock: ident, $payload: ty) => {
            fn $fct(&self) -> Result<Box<$payload>, FileStructureError> {
                match &self.$mock {
                    Some(e) => match e {
                        Ok(b) => Ok(Box::new(*b.clone())),
                        Err(r) => Err(FileStructureError::Mock(r.to_string())),
                    },
                    None => self.dir.$fct(),
                }
            }
        };
    }
    pub(super) use wrap_payload_getter;

    /// Macro to implement a function to mock a payload
    ///
    /// Parameters:
    /// - $fct: The name of the function
    /// - $mock: The name of the mocked structure field to update
    /// - $payload: Type of the payload
    macro_rules! mock_payload {
        ($fct: ident, $mock: ident, $payload: ty) => {
            pub fn $fct(&mut self, data: &Result<&$payload, FileStructureError>) {
                self.$mock = match data {
                    Ok(d) => Some(Ok(Box::new(d.clone().to_owned()))),
                    Err(e) => Some(Err(FileStructureError::Mock(e.to_string()))),
                };
            }
        };
    }
    pub(super) use mock_payload;
}
