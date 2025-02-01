//! Module implementing the data structures of the setup directory

pub mod control_component_code_shares_payload;
pub mod setup_component_verification_data_payload;

use self::{
    control_component_code_shares_payload::ControlComponentCodeSharesPayload,
    setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
};
use super::VerifierSetupDataTrait;
use enum_kinds::EnumKind;

/// Types of the setup directory
/// An enum VerifierSetupDataType is automatically creating
#[derive(Clone, EnumKind)]
#[enum_kind(VerifierSetupDataType)]
pub enum VerifierSetupData {
    SetupComponentVerificationDataPayload(SetupComponentVerificationDataPayload),
    ControlComponentCodeSharesPayload(ControlComponentCodeSharesPayload),
}

impl VerifierSetupDataTrait for VerifierSetupData {
    fn setup_component_verification_data_payload(
        self,
    ) -> Option<SetupComponentVerificationDataPayload> {
        if let VerifierSetupData::SetupComponentVerificationDataPayload(d) = self {
            return Some(d);
        }
        None
    }

    fn control_component_code_shares_payload(self) -> Option<ControlComponentCodeSharesPayload> {
        if let VerifierSetupData::ControlComponentCodeSharesPayload(d) = self {
            return Some(d);
        }
        None
    }
}
