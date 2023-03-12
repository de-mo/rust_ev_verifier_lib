use self::{
    control_component_code_shares_payload::ControlComponentCodeSharesPayload,
    control_component_public_keys_payload::ControlComponentPublicKeysPayload,
    election_event_context_payload::ElectionEventContextPayload,
    encryption_parameters_payload::EncryptionParametersPayload,
    setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
    setup_component_tally_data_payload::SetupComponentTallyDataPayload,
    setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
};
use super::{error::DeserializeError, DataStructureTrait, VerifierDataTrait};

use enum_kinds::EnumKind;

pub mod control_component_code_shares_payload;
pub mod control_component_public_keys_payload;
pub mod election_event_context_payload;
pub mod encryption_parameters_payload;
pub mod setup_component_public_keys_payload;
pub mod setup_component_tally_data_payload;
pub mod setup_component_verification_data_payload;

#[derive(Clone, EnumKind)]
#[enum_kind(VerifierSetupDataType)]
pub enum VerifierSetupData {
    EncryptionParametersPayload(EncryptionParametersPayload),
    ElectionEventContextPayload(ElectionEventContextPayload),
    SetupComponentPublicKeysPayload(SetupComponentPublicKeysPayload),
    ControlComponentPublicKeysPayload(ControlComponentPublicKeysPayload),
    SetupComponentVerificationDataPayload(SetupComponentVerificationDataPayload),
    ControlComponentCodeSharesPayload(ControlComponentCodeSharesPayload),
    SetupComponentTallyDataPayload(SetupComponentTallyDataPayload),
}

impl VerifierSetupDataType {
    pub fn verifier_data_from_json(
        &self,
        s: &String,
    ) -> Result<VerifierSetupData, DeserializeError> {
        match self {
            VerifierSetupDataType::EncryptionParametersPayload => {
                EncryptionParametersPayload::from_json(s)
                    .map(|r| VerifierSetupData::EncryptionParametersPayload(r))
            }
            VerifierSetupDataType::ElectionEventContextPayload => {
                ElectionEventContextPayload::from_json(s)
                    .map(|r| VerifierSetupData::ElectionEventContextPayload(r))
            }
            VerifierSetupDataType::SetupComponentPublicKeysPayload => {
                SetupComponentPublicKeysPayload::from_json(s)
                    .map(|r| VerifierSetupData::SetupComponentPublicKeysPayload(r))
            }
            VerifierSetupDataType::ControlComponentPublicKeysPayload => {
                ControlComponentPublicKeysPayload::from_json(s)
                    .map(|r| VerifierSetupData::ControlComponentPublicKeysPayload(r))
            }
            VerifierSetupDataType::SetupComponentVerificationDataPayload => {
                SetupComponentVerificationDataPayload::from_json(s)
                    .map(|r| VerifierSetupData::SetupComponentVerificationDataPayload(r))
            }
            VerifierSetupDataType::ControlComponentCodeSharesPayload => {
                ControlComponentCodeSharesPayload::from_json(s)
                    .map(|r| VerifierSetupData::ControlComponentCodeSharesPayload(r))
            }
            VerifierSetupDataType::SetupComponentTallyDataPayload => {
                SetupComponentTallyDataPayload::from_json(s)
                    .map(|r| VerifierSetupData::SetupComponentTallyDataPayload(r))
            }
        }
    }
}

impl VerifierDataTrait for VerifierSetupData {
    fn encryption_parameters_payload(&self) -> Option<&EncryptionParametersPayload> {
        if let VerifierSetupData::EncryptionParametersPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn setup_component_public_keys_payload(&self) -> Option<&SetupComponentPublicKeysPayload> {
        if let VerifierSetupData::SetupComponentPublicKeysPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn election_event_context_payload(&self) -> Option<&ElectionEventContextPayload> {
        if let VerifierSetupData::ElectionEventContextPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn setup_component_tally_data_payload(&self) -> Option<&SetupComponentTallyDataPayload> {
        if let VerifierSetupData::SetupComponentTallyDataPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn control_component_public_keys_payload(&self) -> Option<&ControlComponentPublicKeysPayload> {
        if let VerifierSetupData::ControlComponentPublicKeysPayload(d) = self {
            return Some(d);
        }
        None
    }

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
