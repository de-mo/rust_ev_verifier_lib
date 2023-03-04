pub mod structure;
use crate::data_structures::setup::VerifierSetupData;
use crate::data_structures::VerifierData;
use crate::verification::VerificationPeriod;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::error::VerifierError;

pub struct SetupDirectory {
    location: PathBuf,
}

pub struct TallyDirectory {
    location: PathBuf,
}

pub enum VerificationDirectory {
    Setup(SetupDirectory),
    Tally(TallyDirectory),
}

impl SetupDirectory {
    pub fn new(location: &Path) -> SetupDirectory {
        SetupDirectory {
            location: location.to_path_buf(),
        }
    }
}

impl TallyDirectory {
    pub fn new(location: &Path) -> TallyDirectory {
        TallyDirectory {
            location: location.to_path_buf(),
        }
    }
}

impl VerificationDirectory {
    pub fn new(period: VerificationPeriod, location: &Path) -> VerificationDirectory {
        match period {
            VerificationPeriod::Setup => {
                VerificationDirectory::Setup(SetupDirectory::new(location))
            }
            VerificationPeriod::Tally => {
                VerificationDirectory::Tally(TallyDirectory::new(location))
            }
        }
    }

    pub fn get_setup(&self) -> Option<&SetupDirectory> {
        match self {
            VerificationDirectory::Setup(d) => Some(d),
            VerificationDirectory::Tally(_) => None,
        }
    }

    pub fn get_tally(&self) -> Option<&TallyDirectory> {
        match self {
            VerificationDirectory::Setup(_) => None,
            VerificationDirectory::Tally(d) => Some(d),
        }
    }
}

pub trait GetFileName {
    fn get_raw_file_name(&self) -> String;
    fn get_file_name(&self, value: Option<usize>) -> String;
}

impl GetFileName for VerifierSetupData {
    fn get_raw_file_name(&self) -> String {
        let s = match self {
            Self::EncryptionParametersPayload(_) => "encryptionParametersPayload.json",
            Self::ElectionEventContextPayload(_) => "electionEventContextPayload.json",
            Self::SetupComponentPublicKeysPayload(_) => "setupComponentPublicKeysPayload.json",
            Self::ControlComponentPublicKeysPayload(_) => {
                "controlComponentPublicKeysPayload.{}.json"
            }
            Self::SetupComponentVerificationDataPayload(_) => {
                "setupComponentVerificationDataPayload.{}.json"
            }
            Self::ControlComponentCodeSharesPayload(_) => {
                "controlComponentCodeSharesPayload.{}.json"
            }
            Self::SetupComponentTallyDataPayload(_) => "setupComponentTallyDataPayload.json",
        };
        s.to_string()
    }

    fn get_file_name(&self, value: Option<usize>) -> String {
        let s = self.get_raw_file_name();
        match value {
            Some(i) => s.replace("{}", &i.to_string()),
            None => s,
        }
    }
}

impl GetFileName for VerifierData {
    fn get_raw_file_name(&self) -> String {
        match self {
            VerifierData::Setup(t) => t.get_raw_file_name(),
            VerifierData::Tally => todo!(),
        }
    }

    fn get_file_name(&self, value: Option<usize>) -> String {
        match self {
            VerifierData::Setup(t) => t.get_file_name(value),
            VerifierData::Tally => todo!(),
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
                VerifierData::Setup(VerifierSetupData::EncryptionParametersPayload(None))
                    .get_file_name(None)
            )
            .exists());
        assert!(path
            .join(
                VerifierData::Setup(VerifierSetupData::ElectionEventContextPayload(None))
                    .get_file_name(None)
            )
            .exists());
        assert!(path
            .join(
                VerifierData::Setup(VerifierSetupData::SetupComponentPublicKeysPayload(None))
                    .get_file_name(None)
            )
            .exists());
        let path2 = path
            .join("verification_card_sets")
            .join("743f2d0fc9fc412798876d7763f78f1b");
        assert!(path2
            .join(
                VerifierData::Setup(VerifierSetupData::SetupComponentTallyDataPayload(None))
                    .get_file_name(None)
            )
            .exists());
    }

    #[test]
    fn test_setup_groupss_exist() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup");
        assert!(path
            .join(
                VerifierData::Setup(VerifierSetupData::ControlComponentPublicKeysPayload(None))
                    .get_file_name(Some(1))
            )
            .exists());
        let path2 = path
            .join("verification_card_sets")
            .join("743f2d0fc9fc412798876d7763f78f1b");
        assert!(path2
            .join(
                VerifierData::Setup(VerifierSetupData::ControlComponentCodeSharesPayload(None))
                    .get_file_name(Some(1))
            )
            .exists());
        assert!(path2
            .join(
                VerifierData::Setup(VerifierSetupData::SetupComponentVerificationDataPayload(
                    None
                ))
                .get_file_name(Some(1))
            )
            .exists());
    }
}
