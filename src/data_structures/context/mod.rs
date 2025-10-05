// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

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
