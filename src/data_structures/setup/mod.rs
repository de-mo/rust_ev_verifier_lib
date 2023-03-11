use self::{
    control_component_code_shares_payload::ControlComponentCodeSharesPayload,
    control_component_public_keys_payload::ControlComponentPublicKeysPayload,
    election_event_context_payload::ElectionEventContextPayload,
    encryption_parameters_payload::EncryptionParametersPayload,
    setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
    setup_component_tally_data_payload::SetupComponentTallyDataPayload,
    setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
};

use super::{DataStructureTrait, VerifierDataTrait};

use enum_kinds::EnumKind;

pub mod control_component_code_shares_payload;
pub mod control_component_public_keys_payload;
pub mod election_event_context_payload;
pub mod encryption_parameters_payload;
pub mod setup_component_public_keys_payload;
pub mod setup_component_tally_data_payload;
pub mod setup_component_verification_data_payload;

/*
pub enum VerifierSetupDataType {
    EncryptionParametersPayload,
    ElectionEventContextPayload,
    SetupComponentPublicKeysPayload,
    ControlComponentPublicKeysPayload,
    SetupComponentVerificationDataPayload,
    ControlComponentCodeSharesPayload,
    SetupComponentTallyDataPayload,
} */

#[derive(EnumKind)]
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

impl VerifierSetupData {
    /*
    pub fn new_EncryptionParametersPayload() -> Self {
        VerifierSetupDataType::EncryptionParametersPayload(None)
    }
    pub fn new_ElectionEventContextPayload() -> Self {
        VerifierSetupDataType::ElectionEventContextPayload(None)
    }
    pub fn new_SetupComponentPublicKeysPayload() -> Self {
        VerifierSetupDataType::SetupComponentPublicKeysPayload(None)
    }
    pub fn new_ControlComponentPublicKeysPayload() -> Self {
        VerifierSetupDataType::ControlComponentPublicKeysPayload(None)
    }
    pub fn new_SetupComponentVerificationDataPayload() -> Self {
        VerifierSetupDataType::SetupComponentVerificationDataPayload(None)
    }
    pub fn new_ControlComponentCodeSharesPayload() -> Self {
        VerifierSetupDataType::ControlComponentCodeSharesPayload(None)
    }
    pub fn new_SetupComponentTallyDataPayload() -> Self {
        VerifierSetupDataType::SetupComponentTallyDataPayload(None)
    } */
}

impl VerifierSetupDataType {
    pub fn verifier_data_from_json(
        &self,
        s: &String,
    ) -> Result<VerifierSetupData, super::DeserializeError> {
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

/*
impl VerifierSetupDataType {
    pub fn read_data(&self, s: &String) -> Box<dyn DataStructureTrait> {
        todo!()
    }

    fn get_encryption_parameters_payload(&self, s: &String) -> EncryptionParametersPayload {
        self.read_data(s).as_any()
    }
}
 */

/*
impl VerifierDataTrait for VerifierSetupDataType {
    fn get_encryption_parameters_payload(&self) -> Self {
        match self {
            VerifierSetupData::EncryptionParametersPayload(_) => {
                VerifierSetupData::EncryptionParametersPayload(None)
            }
            VerifierSetupData::ElectionEventContextPayload(_) => todo!(),
            VerifierSetupData::SetupComponentPublicKeysPayload(_) => todo!(),
            VerifierSetupData::ControlComponentPublicKeysPayload(_) => todo!(),
            VerifierSetupData::SetupComponentVerificationDataPayload(_) => todo!(),
            VerifierSetupData::ControlComponentCodeSharesPayload(_) => todo!(),
            VerifierSetupData::SetupComponentTallyDataPayload(_) => todo!(),
        }
    }

    fn new_from_json(&self, s: &String) -> Result<Self, super::DeserializeError> {
        match self {
            VerifierSetupData::EncryptionParametersPayload(_) => {
                EncryptionParametersPayload::from_json(s)
                    .map(|r| VerifierSetupData::EncryptionParametersPayload(Some(Box::new(r))))
            }
            VerifierSetupData::ElectionEventContextPayload(_) => todo!(),
            VerifierSetupData::SetupComponentPublicKeysPayload(_) => todo!(),
            VerifierSetupData::ControlComponentPublicKeysPayload(_) => todo!(),
            VerifierSetupData::SetupComponentVerificationDataPayload(_) => todo!(),
            VerifierSetupData::ControlComponentCodeSharesPayload(_) => todo!(),
            VerifierSetupData::SetupComponentTallyDataPayload(_) => todo!(),
        }
    }

    fn is_some(&self) -> bool {
        match self {
            VerifierSetupData::EncryptionParametersPayload(x) => x.is_none(),
            VerifierSetupData::ElectionEventContextPayload(_) => todo!(),
            VerifierSetupData::SetupComponentPublicKeysPayload(_) => todo!(),
            VerifierSetupData::ControlComponentPublicKeysPayload(_) => todo!(),
            VerifierSetupData::SetupComponentVerificationDataPayload(_) => todo!(),
            VerifierSetupData::ControlComponentCodeSharesPayload(_) => todo!(),
            VerifierSetupData::SetupComponentTallyDataPayload(_) => todo!(),
        }
    }

    fn get_encryption_parameters_payload(&self) -> Option<&Box<EncryptionParametersPayload>> {
        if let VerifierSetupData::EncryptionParametersPayload(Some(d)) = self {
            return Some(d);
        }
        None
    }
} */

impl VerifierSetupData {
    /*
    pub fn new_EncryptionParametersPayload() -> Self {
        VerifierSetupDataType::EncryptionParametersPayload(None)
    }
    pub fn new_ElectionEventContextPayload() -> Self {
        VerifierSetupDataType::ElectionEventContextPayload(None)
    }
    pub fn new_SetupComponentPublicKeysPayload() -> Self {
        VerifierSetupDataType::SetupComponentPublicKeysPayload(None)
    }
    pub fn new_ControlComponentPublicKeysPayload() -> Self {
        VerifierSetupDataType::ControlComponentPublicKeysPayload(None)
    }
    pub fn new_SetupComponentVerificationDataPayload() -> Self {
        VerifierSetupDataType::SetupComponentVerificationDataPayload(None)
    }
    pub fn new_ControlComponentCodeSharesPayload() -> Self {
        VerifierSetupDataType::ControlComponentCodeSharesPayload(None)
    }
    pub fn new_SetupComponentTallyDataPayload() -> Self {
        VerifierSetupDataType::SetupComponentTallyDataPayload(None)
    } */
}

impl VerifierDataTrait for VerifierSetupData {
    fn encryption_parameters_payload(&self) -> Option<Box<EncryptionParametersPayload>> {
        if let VerifierSetupData::EncryptionParametersPayload(d) = self {
            return Some(Box::new(d.clone()));
        }
        None
    }

    fn setup_component_public_keys_payload(&self) -> Option<Box<SetupComponentPublicKeysPayload>> {
        if let VerifierSetupData::SetupComponentPublicKeysPayload(d) = self {
            return Some(Box::new(d.clone()));
        }
        None
    }

    fn election_event_context_payload(&self) -> Option<Box<ElectionEventContextPayload>> {
        if let VerifierSetupData::ElectionEventContextPayload(d) = self {
            return Some(Box::new(d.clone()));
        }
        None
    }
}
