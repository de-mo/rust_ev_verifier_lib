//! Module implementing the necessary cryprographic function

pub mod byte_array;
pub mod direct_trust;
pub mod elgamal;
pub mod hashing;
pub mod num_bigint;
pub mod number_theory;
pub mod openssl_wrapper;
pub mod signature;
pub mod zero_knowledge_proof;

pub const GROUP_PARAMETER_P_LENGTH: usize = 3072;
