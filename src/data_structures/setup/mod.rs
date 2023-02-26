use self::{
    control_component_code_shares_payload::ControlComponentCodeSharesPayload,
    control_component_public_keys_payload::ControlComponentPublicKeysPayload,
    election_event_context_payload::ElectionEventContextPayload,
    encryption_parameters_payload::EncryptionParametersPayload,
    setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
    setup_component_tally_data_payload::SetupComponentTallyDataPayload,
    setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
};

use super::verifier_data::{VerifierData, VerifierDataTrait, VerifierDataTraitNew};
use super::VerifierDataType;

pub mod control_component_code_shares_payload;
pub mod control_component_public_keys_payload;
pub mod election_event_context_payload;
pub mod encryption_parameters_payload;
pub mod setup_component_public_keys_payload;
pub mod setup_component_tally_data_payload;
pub mod setup_component_verification_data_payload;

pub enum VerifierSetupDataType {
    EncryptionParametersPayload,
    ElectionEventContextPayload,
    SetupComponentPublicKeysPayload,
    ControlComponentPublicKeysPayload,
    SetupComponentVerificationDataPayload,
    ControlComponentCodeSharesPayload,
    SetupComponentTallyDataPayload,
}

impl VerifierDataTraitNew<EncryptionParametersPayload>
    for VerifierData<EncryptionParametersPayload>
{
    fn new_without_data() -> Self {
        Self::new(
            VerifierDataType::Setup(VerifierSetupDataType::EncryptionParametersPayload),
            None,
        )
    }
}

impl VerifierDataTraitNew<ElectionEventContextPayload>
    for VerifierData<ElectionEventContextPayload>
{
    fn new_without_data() -> Self {
        Self::new(
            VerifierDataType::Setup(VerifierSetupDataType::ElectionEventContextPayload),
            None,
        )
    }
}

impl VerifierDataTraitNew<SetupComponentPublicKeysPayload>
    for VerifierData<SetupComponentPublicKeysPayload>
{
    fn new_without_data() -> Self {
        Self::new(
            VerifierDataType::Setup(VerifierSetupDataType::SetupComponentPublicKeysPayload),
            None,
        )
    }
}

impl VerifierDataTraitNew<ControlComponentPublicKeysPayload>
    for VerifierData<ControlComponentPublicKeysPayload>
{
    fn new_without_data() -> Self {
        Self::new(
            VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload),
            None,
        )
    }
}

impl VerifierDataTraitNew<SetupComponentVerificationDataPayload>
    for VerifierData<SetupComponentVerificationDataPayload>
{
    fn new_without_data() -> Self {
        Self::new(
            VerifierDataType::Setup(VerifierSetupDataType::SetupComponentVerificationDataPayload),
            None,
        )
    }
}

impl VerifierDataTraitNew<ControlComponentCodeSharesPayload>
    for VerifierData<ControlComponentCodeSharesPayload>
{
    fn new_without_data() -> Self {
        Self::new(
            VerifierDataType::Setup(VerifierSetupDataType::ControlComponentCodeSharesPayload),
            None,
        )
    }
}

impl VerifierDataTraitNew<SetupComponentTallyDataPayload>
    for VerifierData<SetupComponentTallyDataPayload>
{
    fn new_without_data() -> Self {
        Self::new(
            VerifierDataType::Setup(VerifierSetupDataType::SetupComponentTallyDataPayload),
            None,
        )
    }
}
