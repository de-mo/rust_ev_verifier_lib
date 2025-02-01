//! Module implementing the data structures of the context directory

pub mod control_component_public_keys_payload;
pub mod election_event_configuration;
pub mod election_event_context_payload;
pub mod setup_component_public_keys_payload;
pub mod setup_component_tally_data_payload;

#[derive(Clone, PartialEq, Eq)]
pub enum VerifierContextDataType {
    ElectionEventContextPayload,
    SetupComponentPublicKeysPayload,
    ControlComponentPublicKeysPayload,
    SetupComponentTallyDataPayload,
    ElectionEventConfiguration,
}
