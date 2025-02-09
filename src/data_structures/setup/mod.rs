//! Module implementing the data structures of the setup directory

pub mod control_component_code_shares_payload;
pub mod setup_component_verification_data_payload;

/// Types of the setup directory
#[derive(Clone, PartialEq, Eq)]
pub enum VerifierSetupDataType {
    SetupComponentVerificationDataPayload,
    ControlComponentCodeSharesPayload,
}
