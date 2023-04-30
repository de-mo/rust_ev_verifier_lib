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

use self::{setup_directory::SetupDirectoryTrait, tally_directory::TallyDirectoryTrait};

/// Type represending a VerificationDirectory
pub struct VerificationDirectory {
    setup: SetupDirectory,
    tally: Option<TallyDirectory>,
}

pub enum FileType {
    Json,
    Xml,
}

pub enum FileReadMode {
    Memory,
    Streaming,
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
    type SetupDirType: SetupDirectoryTrait;
    type TallyDirType: TallyDirectoryTrait;

    /// Unwrap setup and give a reference to S
    ///
    /// panic if type is tally
    fn unwrap_setup(&self) -> &Self::SetupDirType;

    /// Unwrap tally and give a reference to S
    ///
    /// panic if type is seup
    fn unwrap_tally(&self) -> &Self::TallyDirType;
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

impl VerificationDirectoryTrait for VerificationDirectory {
    type SetupDirType = SetupDirectory;
    type TallyDirType = TallyDirectory;

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
            Self::EVotingDecrypt => "evoting-decrypt_*.xml",
            Self::ECH0222 => "eCH-0222_*.xml",
            Self::TallyComponentVotesPayload => "tallyComponentVotesPayload.json",
            Self::TallyComponentShufflePayload => "TallyComponentShufflePayload.json",
            Self::ControlComponentBallotBoxPayload => "controlComponentBallotBoxPayload_{}.json",
            Self::ControlComponentShufflePayload => "controlComponentShufflePayload_{}.json",
        };
        s.to_string()
    }
}

impl GetFileNameTrait for VerifierDataType {
    fn get_raw_file_name(&self) -> String {
        match self {
            VerifierDataType::Setup(t) => t.get_raw_file_name(),
            VerifierDataType::Tally(t) => t.get_raw_file_name(),
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
    fn test_tally_files_exist() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset1")
            .join("tally");
        let path2 = path
            .join("ballot_boxes")
            .join("9a19164550794441b25f7f744f2e91fb");
        assert!(path2
            .join(
                VerifierDataType::Tally(VerifierTallyDataType::TallyComponentVotesPayload)
                    .get_file_name(None)
            )
            .exists());
        assert!(path2
            .join(
                VerifierDataType::Tally(VerifierTallyDataType::TallyComponentShufflePayload)
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
    use super::{
        setup_directory::mock::MockSetupDirectory, tally_directory::mock::MockTallyDirectory, *,
    };

    /// Mock for [VerificationDirectory]
    pub struct MockVerificationDirectory {
        setup: MockSetupDirectory,
        tally: Option<MockTallyDirectory>,
    }

    impl VerificationDirectoryTrait for MockVerificationDirectory {
        type SetupDirType = MockSetupDirectory;
        type TallyDirType = MockTallyDirectory;
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
                        Err(r) => create_result_with_error!(r.kind().clone(), r.message()),
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
                    Err(e) => Some(create_result_with_error!(e.kind().clone(), e.message())),
                };
            }
        };
    }
    pub(super) use mock_payload;
}
