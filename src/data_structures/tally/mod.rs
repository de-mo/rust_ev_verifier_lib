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
