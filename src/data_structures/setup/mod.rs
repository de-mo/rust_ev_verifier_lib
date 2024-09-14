//! Module implementing the data structures of the setup directory

pub mod control_component_code_shares_payload;
pub mod setup_component_verification_data_payload;

use self::{
    control_component_code_shares_payload::ControlComponentCodeSharesPayload,
    setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
};
use super::{DataStructureError, VerifierDataDecode, VerifierSetupDataTrait};
use crate::file_structure::{
    file::File, FileReadMode, FileType, GetFileReadMode, GetFileTypeTrait,
};
use enum_kinds::EnumKind;

/// Types of the setup directory
/// An enum VerifierSetupDataType is automatically creating
#[derive(Clone, EnumKind)]
#[enum_kind(VerifierSetupDataType)]
pub enum VerifierSetupData {
    SetupComponentVerificationDataPayload(SetupComponentVerificationDataPayload),
    ControlComponentCodeSharesPayload(ControlComponentCodeSharesPayload),
}

impl GetFileReadMode for VerifierSetupDataType {
    fn get_file_read_mode(&self) -> FileReadMode {
        match self {
            Self::SetupComponentVerificationDataPayload => FileReadMode::Memory,
            Self::ControlComponentCodeSharesPayload => FileReadMode::Memory,
        }
    }
}

impl GetFileTypeTrait for VerifierSetupDataType {
    fn get_file_type(&self) -> FileType {
        match self {
            Self::SetupComponentVerificationDataPayload => FileType::Json,
            Self::ControlComponentCodeSharesPayload => FileType::Json,
        }
    }
}

/*
impl VerifierSetupDataType {
    /// Get the type of the file for the [VerifierSetupData]


    /// Get the read mode of the file for the [VerifierSetupData]


    /// Read from String as json or xml
    ///
    /// All the types have to oimplement the trait [VerifierDataDecode]
    pub fn verifier_data_from_file(
        &self,
        f: &File,
    ) -> Result<VerifierSetupData, DataStructureError> {
        match self {
            VerifierSetupDataType::SetupComponentVerificationDataPayload => {
                SetupComponentVerificationDataPayload::from_file(
                    f,
                    &self.get_file_type(),
                    &self.get_file_read_mode(),
                )
                .map(VerifierSetupData::SetupComponentVerificationDataPayload)
            }
            VerifierSetupDataType::ControlComponentCodeSharesPayload => {
                ControlComponentCodeSharesPayload::from_file(
                    f,
                    &self.get_file_type(),
                    &self.get_file_read_mode(),
                )
                .map(VerifierSetupData::ControlComponentCodeSharesPayload)
            }
        }
    }
} */

impl VerifierSetupDataTrait for VerifierSetupData {
    fn setup_component_verification_data_payload(
        &self,
    ) -> Option<&SetupComponentVerificationDataPayload> {
        if let VerifierSetupData::SetupComponentVerificationDataPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn control_component_code_shares_payload(&self) -> Option<&ControlComponentCodeSharesPayload> {
        if let VerifierSetupData::ControlComponentCodeSharesPayload(d) = self {
            return Some(d);
        }
        None
    }
}
