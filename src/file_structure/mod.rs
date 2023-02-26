pub mod structure;
use crate::data_structures::setup::VerifierSetupDataType;
use crate::data_structures::VerifierDataType;
use std::fmt::Display;

use crate::error::VerifierError;

pub trait GetFileName {
    fn get_file_name(&self) -> String;
}

impl GetFileName for VerifierSetupDataType {
    fn get_file_name(&self) -> String {
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
}

impl GetFileName for VerifierDataType {
    fn get_file_name(&self) -> String {
        match self {
            VerifierDataType::Setup(t) => t.get_file_name(),
            VerifierDataType::Tally => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStructureErrorType {
    FileError,
    DataError,
}

impl Display for FileStructureErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::FileError => "FileError",
            Self::DataError => "DataError",
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
                    .get_file_name()
            )
            .exists());
        assert!(path
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::ElectionEventContextPayload)
                    .get_file_name()
            )
            .exists());
        assert!(path
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::SetupComponentPublicKeysPayload)
                    .get_file_name()
            )
            .exists());
        let path2 = path
            .join("verification_card_sets")
            .join("743f2d0fc9fc412798876d7763f78f1b");
        assert!(path2
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::SetupComponentTallyDataPayload)
                    .get_file_name()
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
                VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload)
                    .get_file_name()
                    .replace("{}", "1")
            )
            .exists());
        let path2 = path
            .join("verification_card_sets")
            .join("743f2d0fc9fc412798876d7763f78f1b");
        assert!(path2
            .join(
                VerifierDataType::Setup(VerifierSetupDataType::ControlComponentCodeSharesPayload)
                    .get_file_name()
                    .replace("{}", "1")
            )
            .exists());
        assert!(path2
            .join(
                VerifierDataType::Setup(
                    VerifierSetupDataType::SetupComponentVerificationDataPayload
                )
                .get_file_name()
                .replace("{}", "1")
            )
            .exists());
    }
}
