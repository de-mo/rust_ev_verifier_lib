pub mod control_component_ballot_box_payload;
pub mod control_component_shuffle_payload;
pub mod e_voting_decrypt;
pub mod ech_0110;
pub mod ech_0222;
pub mod tally_component_shuffle_payload;
pub mod tally_component_votes_payload;

use self::{
    control_component_ballot_box_payload::ControlComponentBallotBoxPayload,
    control_component_shuffle_payload::ControlComponentShufflePayload,
    e_voting_decrypt::EVotingDecrypt, ech_0110::ECH0110, ech_0222::ECH0222,
    tally_component_shuffle_payload::TallyComponentShufflePayload,
    tally_component_votes_payload::TallyComponentVotesPayload,
};
use super::VerifierTallyDataTrait;
use enum_kinds::EnumKind;

#[derive(Clone, EnumKind)]
#[enum_kind(VerifierTallyDataType)]
pub enum VerifierTallyData {
    EVotingDecrypt(EVotingDecrypt),
    ECH0110(ECH0110),
    ECH0222(ECH0222),
    TallyComponentVotesPayload(TallyComponentVotesPayload),
    TallyComponentShufflePayload(TallyComponentShufflePayload),
    ControlComponentBallotBoxPayload(ControlComponentBallotBoxPayload),
    ControlComponentShufflePayload(ControlComponentShufflePayload),
}

impl VerifierTallyDataTrait for VerifierTallyData {
    fn e_voting_decrypt(&self) -> Option<&EVotingDecrypt> {
        if let VerifierTallyData::EVotingDecrypt(d) = self {
            return Some(d);
        }
        None
    }

    fn ech_0110(&self) -> Option<&ECH0110> {
        if let VerifierTallyData::ECH0110(d) = self {
            return Some(d);
        }
        None
    }

    fn ech_0222(&self) -> Option<&ECH0222> {
        if let VerifierTallyData::ECH0222(d) = self {
            return Some(d);
        }
        None
    }
    fn tally_component_votes_payload(&self) -> Option<&TallyComponentVotesPayload> {
        if let VerifierTallyData::TallyComponentVotesPayload(d) = self {
            return Some(d);
        }
        None
    }
    fn tally_component_shuffle_payload(&self) -> Option<&TallyComponentShufflePayload> {
        if let VerifierTallyData::TallyComponentShufflePayload(d) = self {
            return Some(d);
        }
        None
    }
    fn control_component_ballot_box_payload(&self) -> Option<&ControlComponentBallotBoxPayload> {
        if let VerifierTallyData::ControlComponentBallotBoxPayload(d) = self {
            return Some(d);
        }
        None
    }
    fn control_component_shuffle_payload(&self) -> Option<&ControlComponentShufflePayload> {
        if let VerifierTallyData::ControlComponentShufflePayload(d) = self {
            return Some(d);
        }
        None
    }
}
