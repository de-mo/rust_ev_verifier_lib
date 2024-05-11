use thiserror::Error;

#[derive(Error, Debug)]
pub enum MixDecOfflineError {}

pub fn verify_mix_dec_offline() -> Result<bool, MixDecOfflineError> {
    todo!()
}
