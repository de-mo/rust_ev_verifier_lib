//! Module to implement the signature and the verification of the signature of
//! an object.

use std::fmt::Display;
use std::path::Path;

use super::{byte_array::ByteArray, hashing::RecursiveHashable};
use super::{
    direct_trust::{CertificateAuthority, DirectTrust},
    openssl_wrapper::signature::verify,
};
use crate::data_structures::SignatureTrait;
use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureErrorType {
    DirectTrust,
    Validation,
}

impl Display for SignatureErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DirectTrust => "Direct Trust Error",
            Self::Validation => "Validation Error",
        };
        write!(f, "{s}")
    }
}

pub type SignatureError = VerifierError<SignatureErrorType>;

fn verifiy_signature_impl(
    location: &Path,
    authority_id: &CertificateAuthority,
    hashable: &RecursiveHashable,
    context_data: &RecursiveHashable,
    signature: &ByteArray,
) -> Result<bool, SignatureError> {
    let dt = DirectTrust::new(location, authority_id).map_err(|e| {
        create_verifier_error!(SignatureErrorType::DirectTrust, "Error reading keystore", e)
    })?;
    let cert = dt.signing_certificate();
    let time_ok = cert.is_valid_time().map_err(|e| {
        create_verifier_error!(SignatureErrorType::DirectTrust, "Error testing time", e)
    })?;
    if !time_ok {
        return create_result_with_error!(SignatureErrorType::Validation, "Time is not valide");
    }
    let pkey = cert.get_public_key().map_err(|e| {
        create_verifier_error!(
            SignatureErrorType::DirectTrust,
            "Error reading public key",
            e
        )
    })?;
    let h = RecursiveHashable::Composite(vec![hashable.to_owned(), context_data.to_owned()])
        .recursive_hash();
    Ok(verify(pkey.as_ref(), &h, signature))
}

/// Trait that must be implemented for each object
/// implementing RecursiveHashable
pub trait VerifiySignatureTrait<'a>
where
    Self: 'a + SignatureTrait,
    RecursiveHashable: From<&'a Self>,
{
    /// Get the context data of the object according to the specifications
    fn get_context_data(&self) -> RecursiveHashable;

    /// Get the Certificate Authority to the specifications
    fn get_certificate_authority(&self) -> CertificateAuthority;

    /// Verfiy the signature according to the specifications
    fn verifiy_signature(&'a self, location: &Path) -> Result<bool, SignatureError> {
        verifiy_signature_impl(
            location,
            &self.get_certificate_authority(),
            &RecursiveHashable::from(self),
            &self.get_context_data(),
            &self.get_signature(),
        )
    }
}
