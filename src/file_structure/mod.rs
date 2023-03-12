pub mod directory;
pub mod file;
pub mod file_group;
pub mod setup_directory;
pub mod tally_directory;
use crate::data_structures::setup::VerifierSetupDataType;
use crate::data_structures::VerifierDataType;
use std::fmt::Display;

use crate::error::VerifierError;

pub trait GetFileName {
    fn get_raw_file_name(&self) -> String;
    fn get_file_name(&self, value: Option<usize>) -> String;
}

impl GetFileName for VerifierSetupDataType {
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

impl GetFileName for VerifierDataType {
    fn get_raw_file_name(&self) -> String {
        match self {
            VerifierDataType::Setup(t) => t.get_raw_file_name(),
            VerifierDataType::Tally(_) => todo!(),
        }
    }

    fn get_file_name(&self, value: Option<usize>) -> String {
        match self {
            VerifierDataType::Setup(t) => t.get_file_name(value),
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
            .join("743f2d0fc9fc412798876d7763f78f1b");
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
            .join("743f2d0fc9fc412798876d7763f78f1b");
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
