use thiserror::Error;

#[derive(Error, Debug)]
pub enum VotingClientProofError {}

pub fn verify_voting_client_proofs() -> Result<bool, VotingClientProofError> {
    todo!()
}
