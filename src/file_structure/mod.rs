//! Module implementing the structure of files and directories
//! to collect data for the verifications
//!
pub mod file;
pub mod file_group;
pub mod setup_directory;
pub mod tally_directory;

use crate::{
    data_structures::{
        setup::VerifierSetupDataType, tally::VerifierTallyDataType, VerifierDataType,
    },
    error::VerifierError,
    verification::VerificationPeriod,
};
use setup_directory::SetupDirectory;
use std::{fmt::Display, path::Path};
use tally_directory::TallyDirectory;

use self::{
    setup_directory::{SetupDirectoryTrait, VCSDirectory, VCSDirectoryTrait},
    tally_directory::{BBDirectory, BBDirectoryTrait, TallyDirectoryTrait},
};

/// Type represending a VerificationDirectory
pub struct VerificationDirectory {
    setup: SetupDirectory,
    tally: Option<TallyDirectory>,
}

pub enum FileType {
    Json,
    Xml,
}

/// Trait defining functions to get the filename
pub trait GetFileNameTrait {
    /// Get the file name as it is defiened
    fn get_raw_file_name(&self) -> String;

    /// Get the filename injecting the number (if given)
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
/// ```rust
/// fn fn_verification<
///     B: BBDirectoryTrait,
///     V: VCSDirectoryTrait,
///     S: SetupDirectoryTrait<V>,
///     T: TallyDirectoryTrait<B>,
/// >(
///     dir: &dyn VerificationDirectoryTrait<B, V, S, T>,
///     result: &mut VerificationResult,
/// ) {
///     ...
/// }
/// ```
///
/// All the helpers functions have also to be defined with traits and not with the structs. Then it
/// is possible to mock the data
pub trait VerificationDirectoryTrait<B, V, S, T>
where
    V: VCSDirectoryTrait,
    B: BBDirectoryTrait,
    S: SetupDirectoryTrait<V>,
    T: TallyDirectoryTrait<B>,
{
    /// Unwrap setup and give a reference to S
    ///
    /// panic if type is tally
    fn unwrap_setup(&self) -> &S;

    /// Unwrap tally and give a reference to S
    ///
    /// panic if type is seup
    fn unwrap_tally(&self) -> &T;
}

impl VerificationDirectory {
    /// Create a new VerificationDirectory
    pub fn new(period: &VerificationPeriod, location: &Path) -> Self {
        match period {
            VerificationPeriod::Setup => VerificationDirectory {
                setup: SetupDirectory::new(location),
                tally: None,
            },
            VerificationPeriod::Tally => VerificationDirectory {
                setup: SetupDirectory::new(location),
                tally: Some(TallyDirectory::new(location)),
            },
        }
    }

    /// Is setup
    pub fn is_setup(&self) -> bool {
        self.tally.is_none()
    }

    /// Is tally
    pub fn is_tally(&self) -> bool {
        !self.is_setup()
    }
}

impl VerificationDirectoryTrait<BBDirectory, VCSDirectory, SetupDirectory, TallyDirectory>
    for VerificationDirectory
{
    /// Unwrap setup and give a reference to S
    ///
    /// panic if type is tally
    fn unwrap_setup(&self) -> &SetupDirectory {
        &self.setup
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

impl GetFileNameTrait for VerifierSetupDataType {
    fn get_raw_file_name(&self) -> String {
        let s = match self {
            Self::EncryptionParametersPayload => "encryptionParametersPayload.json",
            Self::ElectionEventContextPayload => "electionEventContextPayload.json",
            Self::SetupComponentPublicKeysPayload => "setupComponentPublicKeysPayload.json",
            Self::ControlComponentPublicKeysPayload => "controlComponentPublicKeysPayload.{}.json",
            Self::SetupComponentVerificationDataPayload => {
                "setupComponentVerificationDataPayload.{}.json"
            }
            Self::ControlComponentCodeSharesPayload => "controlComponentCodeSharesPayload.{}.json",
            Self::SetupComponentTallyDataPayload => "setupComponentTallyDataPayload.json",
            Self::ElectionEventConfiguration => "configuration-anonymized.xml",
        };
        s.to_string()
    }
}

impl GetFileNameTrait for VerifierTallyDataType {
    fn get_raw_file_name(&self) -> String {
        let s = match self {
            Self::ECH0110 => "eCH-0110_*.xml",
            Self::EVotingDecrypt => "evoting_decrypt_*.xml",
            Self::ECH0222 => "eCH-0222_*.xml",
            Self::TallyComponentVotesPayload => "tallyComponentVotesPayload.json",
            Self::TallyComponentShufflePayload => "TallyComponentShufflePayload.json",
        };
        s.to_string()
    }
}

impl GetFileNameTrait for VerifierDataType {
    fn get_raw_file_name(&self) -> String {
        match self {
            VerifierDataType::Setup(t) => t.get_raw_file_name(),
            VerifierDataType::Tally(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStructureErrorType {
    FileError,
    DataError,
    IsNotDir,
}

impl Display for FileStructureErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::FileError => "FileError",
            Self::DataError => "DataError",
            Self::IsNotDir => "Not Directory",
        };
        write!(f, "{s}")
    }
}

type FileStructureError = VerifierError<FileStructureErrorType>;

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_setup_files_exist() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup");
        assert!(path
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::EncryptionParametersPayload)
                    .get_file_name(None)
            )
            .exists());
        assert!(path
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::ElectionEventContextPayload)
                    .get_file_name(None)
            )
            .exists());
        assert!(path
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::SetupComponentPublicKeysPayload)
                    .get_file_name(None)
            )
            .exists());
        let path2 = path
            .join("verification_card_sets")
            .join("7e8ce00c2c164c268c11cfa7066e3d9f");
        assert!(path2
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::SetupComponentTallyDataPayload)
                    .get_file_name(None)
            )
            .exists());
    }

    #[test]
    fn test_setup_groups_exist() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup");
        assert!(path
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload)
                    .get_file_name(Some(1))
            )
            .exists());
        let path2 = path
            .join("verification_card_sets")
            .join("7e8ce00c2c164c268c11cfa7066e3d9f");
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
pub mod mock {
    //! Module defining mocking structure for [VerificationDirectory]
    //!
    //! Example of usage:
    //! ```rust
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
    use super::setup_directory::mock::{MockSetupDirectory, MockVCSDirectory};
    use super::tally_directory::mock::{MockBBDirectory, MockTallyDirectory};
    use super::*;

    /// Mock for [VerificationDirectory]
    pub struct MockVerificationDirectory {
        setup: MockSetupDirectory,
        tally: Option<MockTallyDirectory>,
    }

    impl
        VerificationDirectoryTrait<
            MockBBDirectory,
            MockVCSDirectory,
            MockSetupDirectory,
            MockTallyDirectory,
        > for MockVerificationDirectory
    {
        fn unwrap_setup(&self) -> &MockSetupDirectory {
            &self.setup
        }

        fn unwrap_tally(&self) -> &MockTallyDirectory {
            match &self.tally {
                Some(t) => t,
                None => panic!("called `unwrap_tally()` on a `Setup` value"),
            }
        }
    }

    impl MockVerificationDirectory {
        /// Create a new [MockVerificationDirectory]
        pub fn new(period: &VerificationPeriod, location: &Path) -> Self {
            match period {
                VerificationPeriod::Setup => MockVerificationDirectory {
                    setup: MockSetupDirectory::new(location),
                    tally: None,
                },
                VerificationPeriod::Tally => MockVerificationDirectory {
                    setup: MockSetupDirectory::new(location),
                    tally: Some(MockTallyDirectory::new(location)),
                },
            }
        }

        /// Unwrap [MockSetupDirectory] as mutable
        pub fn unwrap_setup_mut(&mut self) -> &mut MockSetupDirectory {
            &mut self.setup
        }

        /// Unwrap [TallyDirectory] as mutable
        pub fn unwrap_tally_mut(&mut self) -> &mut MockTallyDirectory {
            match &mut self.tally {
                Some(t) => t,
                None => panic!("called `unwrap_tally()` on a `Setup` value"),
            }
        }
    }
}
