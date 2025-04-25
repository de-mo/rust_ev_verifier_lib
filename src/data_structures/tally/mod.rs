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

pub mod control_component_ballot_box_payload;
pub mod control_component_shuffle_payload;
pub mod e_voting_decrypt;
pub mod ech_0110;
pub mod ech_0222;
pub mod tally_component_shuffle_payload;
pub mod tally_component_votes_payload;
mod verifiable_shuffle;

#[derive(Clone, PartialEq, Eq)]
pub enum VerifierTallyDataType {
    EVotingDecrypt,
    ECH0110,
    ECH0222,
    TallyComponentVotesPayload,
    TallyComponentShufflePayload,
    ControlComponentBallotBoxPayload,
    ControlComponentShufflePayload,
}
