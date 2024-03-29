//! Module implementing the data structures of the context directory

pub mod control_component_public_keys_payload;
pub mod election_event_configuration;
pub mod election_event_context_payload;
pub mod setup_component_public_keys_payload;
pub mod setup_component_tally_data_payload;

use self::{
    control_component_public_keys_payload::ControlComponentPublicKeysPayload,
    election_event_configuration::ElectionEventConfiguration,
    election_event_context_payload::ElectionEventContextPayload,
    setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
    setup_component_tally_data_payload::SetupComponentTallyDataPayload,
};
use super::{VerifierContextDataTrait, VerifierDataDecode};
use crate::file_structure::{file::File, FileReadMode, FileType};
use enum_kinds::EnumKind;

/// Types of the context directory
/// An enum VerifierContextDataType is automatically creating
#[derive(Clone, EnumKind)]
#[enum_kind(VerifierContextDataType)]
pub enum VerifierContextData {
    ElectionEventContextPayload(ElectionEventContextPayload),
    SetupComponentPublicKeysPayload(SetupComponentPublicKeysPayload),
    ControlComponentPublicKeysPayload(ControlComponentPublicKeysPayload),
    SetupComponentTallyDataPayload(SetupComponentTallyDataPayload),
    ElectionEventConfiguration(ElectionEventConfiguration),
}

impl VerifierContextDataType {
    /// Get the type of the file for the [VerifierContextData]
    pub fn get_file_type(&self) -> FileType {
        match self {
            Self::ElectionEventContextPayload => FileType::Json,
            Self::SetupComponentPublicKeysPayload => FileType::Json,
            Self::ControlComponentPublicKeysPayload => FileType::Json,
            Self::SetupComponentTallyDataPayload => FileType::Json,
            Self::ElectionEventConfiguration => FileType::Xml,
        }
    }

    /// Get the read mode of the file for the [VerifierContextData]
    pub fn get_file_read_mode(&self) -> FileReadMode {
        match self {
            Self::ElectionEventContextPayload => FileReadMode::Memory,
            Self::SetupComponentPublicKeysPayload => FileReadMode::Memory,
            Self::ControlComponentPublicKeysPayload => FileReadMode::Memory,
            Self::SetupComponentTallyDataPayload => FileReadMode::Memory,
            Self::ElectionEventConfiguration => FileReadMode::Streaming,
        }
    }

    /// Read from String as json or xml
    ///
    /// All the types have to oimplement the trait [VerifierDataDecode]
    pub fn verifier_data_from_file(&self, f: &File) -> anyhow::Result<VerifierContextData> {
        match self {
            VerifierContextDataType::ElectionEventContextPayload => {
                ElectionEventContextPayload::from_file(
                    f,
                    &self.get_file_type(),
                    &self.get_file_read_mode(),
                )
                .map(VerifierContextData::ElectionEventContextPayload)
            }
            VerifierContextDataType::SetupComponentPublicKeysPayload => {
                SetupComponentPublicKeysPayload::from_file(
                    f,
                    &self.get_file_type(),
                    &self.get_file_read_mode(),
                )
                .map(VerifierContextData::SetupComponentPublicKeysPayload)
            }
            VerifierContextDataType::ControlComponentPublicKeysPayload => {
                ControlComponentPublicKeysPayload::from_file(
                    f,
                    &self.get_file_type(),
                    &self.get_file_read_mode(),
                )
                .map(VerifierContextData::ControlComponentPublicKeysPayload)
            }
            VerifierContextDataType::SetupComponentTallyDataPayload => {
                SetupComponentTallyDataPayload::from_file(
                    f,
                    &self.get_file_type(),
                    &self.get_file_read_mode(),
                )
                .map(VerifierContextData::SetupComponentTallyDataPayload)
            }
            VerifierContextDataType::ElectionEventConfiguration => {
                ElectionEventConfiguration::from_file(
                    f,
                    &self.get_file_type(),
                    &self.get_file_read_mode(),
                )
                .map(VerifierContextData::ElectionEventConfiguration)
            }
        }
    }
}

impl VerifierContextDataTrait for VerifierContextData {
    fn setup_component_public_keys_payload(&self) -> Option<&SetupComponentPublicKeysPayload> {
        if let VerifierContextData::SetupComponentPublicKeysPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn election_event_context_payload(&self) -> Option<&ElectionEventContextPayload> {
        if let VerifierContextData::ElectionEventContextPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn setup_component_tally_data_payload(&self) -> Option<&SetupComponentTallyDataPayload> {
        if let VerifierContextData::SetupComponentTallyDataPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn control_component_public_keys_payload(&self) -> Option<&ControlComponentPublicKeysPayload> {
        if let VerifierContextData::ControlComponentPublicKeysPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn election_event_configuration(&self) -> Option<&ElectionEventConfiguration> {
        if let VerifierContextData::ElectionEventConfiguration(d) = self {
            return Some(d);
        }
        None
    }
}
