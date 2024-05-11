mod mix_dec_offline;
mod voting_client_proofs;

use thiserror::Error;

pub use mix_dec_offline::verify_mix_dec_offline;
pub use voting_client_proofs::verify_voting_client_proofs;

// enum representing the errors during the algorithms for Mix Offline
#[derive(Error, Debug)]
pub enum MixDecOfflineError {
    #[error(transparent)]
    VotingClientProofError(#[from] voting_client_proofs::VotingClientProofError),
    #[error(transparent)]
    MixDecOfflineError(#[from] mix_dec_offline::MixDecOfflineError),
}
