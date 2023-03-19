use std::fmt::Display;

use super::direct_trust::{CertificateAuthority, Keystore};
use super::{byte_array::ByteArray, hashing::RecursiveHashable};
use crate::data_structures::SignatureTrait;
use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureErrorType {
    PublicKey,
}

impl Display for SignatureErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::PublicKey => "Public Key",
        };
        write!(f, "{s}")
    }
}

pub type SignatureError = VerifierError<SignatureErrorType>;

fn verifiy_signature(
    keystore: &Keystore,
    authority_id: &CertificateAuthority,
    hashable: &RecursiveHashable,
    context_data: &RecursiveHashable,
    signature: &ByteArray,
) -> Result<bool, SignatureError> {
    let pkey = match keystore.get_public_key(authority_id) {
        Ok(pk) => pk,
        Err(e) => {
            return create_result_with_error!(SignatureErrorType::PublicKey, "Error reading PK", e)
        }
    };
    todo!()
}

pub trait VerifiySignatureTrait<'a>
where
    Self: 'a + SignatureTrait,
    RecursiveHashable: From<&'a Self>,
{
    fn get_context_data(&self) -> RecursiveHashable;
    fn get_certificate_authority(&self) -> CertificateAuthority;
    fn verifiy_signature(&'a self, keystore: &Keystore) -> Result<bool, SignatureError> {
        verifiy_signature(
            keystore,
            &self.get_certificate_authority(),
            &RecursiveHashable::from(self),
            &self.get_context_data(),
            &self.get_signature(),
        )
    }
}
