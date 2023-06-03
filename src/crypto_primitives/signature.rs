//! Module to implement the signature and the verification of the signature of
//! an object.

use super::{byte_array::ByteArray, hashing::HashableMessage};
use super::{
    direct_trust::{CertificateAuthority, DirectTrust},
    openssl_wrapper::verify,
};
use crate::error::{create_result_with_error, create_verifier_error, VerifierError};
use std::fmt::Display;
use std::path::Path;

/// Trait that must be implemented for each object implementing a signature to be verified
///
/// [HashableMessage] has to implement the trait From<&...>
pub trait VerifiySignatureTrait<'a>
where
    Self: 'a,
    HashableMessage<'a>: From<&'a Self> + From<&'a str>,
{
    /// Get the context data of the object according to the specifications
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>>;

    /// Get the context data of the object according to the context data
    fn get_context_hashable(&'a self) -> HashableMessage {
        if self.get_context_data().len() == 1 {
            return self.get_context_data()[0].clone();
        }
        HashableMessage::from(self.get_context_data())
    }

    /// Get the Certificate Authority to the specifications
    fn get_certificate_authority(&self) -> CertificateAuthority;

    /// Get the signature of the object
    fn get_signature(&self) -> ByteArray;

    /// Verfiy the signature according to the specifications of Verifier
    fn verifiy_signature(&'a self, location: &Path) -> Result<bool, SignatureError> {
        let dt = DirectTrust::new(location, &self.get_certificate_authority()).map_err(|e| {
            create_verifier_error!(SignatureErrorType::DirectTrust, "Error reading keystore", e)
        })?;
        let cert = dt.signing_certificate();
        let time_ok = cert.is_valid_time().map_err(|e| {
            create_verifier_error!(SignatureErrorType::DirectTrust, "Error testing time", e)
        })?;
        if !time_ok {
            return create_result_with_error!(SignatureErrorType::Validation, "Time is not valid");
        }
        let pkey = cert.get_public_key().map_err(|e| {
            create_verifier_error!(
                SignatureErrorType::DirectTrust,
                "Error reading public key",
                e
            )
        })?;
        verify(
            pkey.as_ref(),
            &HashableMessage::from(self),
            &self.get_context_hashable(),
            &self.get_signature(),
        )
        .map_err(|e| create_verifier_error!(SignatureErrorType::Validation, "Error verfying", e))
    }
}

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
