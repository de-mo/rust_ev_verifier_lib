//! Module implementing the structure of files and directories
//! to collect data for the verifications
//!
pub(crate) mod context_directory;
pub(crate) mod file;
pub(crate) mod file_group;
#[cfg(test)]
pub(crate) mod mock;
pub(crate) mod setup_directory;
pub(crate) mod tally_directory;

pub use self::{
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
}

//#[derive(Clone)]
/// Type represending a VerificationDirectory (subdirectory context and setup or tally)
pub struct VerificationDirectory {
    context: ContextDirectory,
    setup: Option<SetupDirectory>,
    tally: Option<TallyDirectory>,
}

/// Enum to define the type of the file (Json or Xml)
#[derive(Debug, Clone, PartialEq, Eq)]
enum FileType {
    Json,
    Xml,
}

/// Enum representing the mode to read a fie.
#[derive(Debug, Clone, PartialEq, Eq)]
enum FileReadMode {
    /// The data will be loaded in memory each time
    Memory,
    /// The data will be streamed
    Streaming,
    /// The data will be loaded once in memory an be chached
    Cache,
}

/// Trait defining functions to get the filename
trait GetFileNameTrait {
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
    /// ```ignore
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

    fn path(&self) -> &Path;
    fn path_to_string(&self) -> String {
        self.path().to_str().unwrap().to_string()
    }
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
    pub fn is_setup(&self) -> bool {
        self.setup.is_some()
    }

    /// Is tally
    pub fn is_tally(&self) -> bool {
        self.tally.is_some()
    }

    /// Are the entries valid
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

    fn path(&self) -> &Path {
        self.context.location().parent().unwrap()
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

impl From<&VerifierContextDataType> for FileReadMode {
    fn from(value: &VerifierContextDataType) -> Self {
        match value {
            VerifierContextDataType::ElectionEventContextPayload => FileReadMode::Cache,
            VerifierContextDataType::SetupComponentPublicKeysPayload => FileReadMode::Memory,
            VerifierContextDataType::ControlComponentPublicKeysPayload => FileReadMode::Memory,
            VerifierContextDataType::SetupComponentTallyDataPayload => FileReadMode::Memory,
            VerifierContextDataType::ElectionEventConfiguration => FileReadMode::Streaming,
        }
    }
}

impl From<&VerifierContextDataType> for FileType {
    fn from(value: &VerifierContextDataType) -> Self {
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

impl From<&VerifierSetupDataType> for FileReadMode {
    fn from(value: &VerifierSetupDataType) -> Self {
        match value {
            VerifierSetupDataType::SetupComponentVerificationDataPayload => FileReadMode::Memory,
            VerifierSetupDataType::ControlComponentCodeSharesPayload => FileReadMode::Memory,
        }
    }
}

impl From<&VerifierSetupDataType> for FileType {
    fn from(value: &VerifierSetupDataType) -> Self {
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

impl From<&VerifierTallyDataType> for FileReadMode {
    fn from(value: &VerifierTallyDataType) -> Self {
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

impl From<&VerifierTallyDataType> for FileType {
    fn from(value: &VerifierTallyDataType) -> Self {
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

impl From<&VerifierDataType> for FileReadMode {
    fn from(value: &VerifierDataType) -> Self {
        match value {
            VerifierDataType::Context(t) => FileReadMode::from(t),
            VerifierDataType::Setup(t) => FileReadMode::from(t),
            VerifierDataType::Tally(t) => FileReadMode::from(t),
        }
    }
}

impl From<&VerifierDataType> for FileType {
    fn from(value: &VerifierDataType) -> Self {
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
        test_ballot_box_one_vote_path, test_context_verification_card_set_path,
        test_datasets_context_path, test_setup_verification_card_set_path,
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
        let path2 = test_ballot_box_one_vote_path();
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
