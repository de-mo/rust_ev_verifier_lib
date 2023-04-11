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

impl VerificationDirectory {
    /// Create a new VerificationDirectory
    pub fn new(period: VerificationPeriod, location: &Path) -> Self {
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

    /// Unwrap setup and give a reference to S
    ///
    /// panic if type is tally
    pub fn unwrap_setup(&self) -> &SetupDirectory {
        &self.setup
    }

    /// Unwrap tally and give a reference to S
    ///
    /// panic if type is seup
    pub fn unwrap_tally(&self) -> &TallyDirectory {
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
            Self::ECH0110 => "eCH-0110_*.json",
            Self::EVotingDecrypt => "evoting_decrypt_*.json",
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
