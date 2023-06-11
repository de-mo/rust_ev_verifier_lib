//! Module to implement the signature and the verification of the signature of
//! an object.

use super::{byte_array::ByteArray, hashing::HashableMessage};
use super::{
    direct_trust::{CertificateAuthority, DirectTrust, DirectTrustError},
    openssl_wrapper::{verify, OpensslError},
};
use std::path::Path;
use thiserror::Error;

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
        //        let dt = DirectTrustCertificate::new(location, &self.get_certificate_authority());
        let dtc = DirectTrust::new(location)
            .map_err(SignatureError::Keystore)?
            .certificate(&self.get_certificate_authority())
            .map_err(SignatureError::Keystore)?;
        let cert = dtc.signing_certificate();
        //dt.signing_certificate();
        let time_ok = cert
            .is_valid_time()
            .map_err(|e| SignatureError::Certificate {
                name: String::from(&self.get_certificate_authority()),
                error: e,
                action: "validating time".to_string(),
            })?;
        if !time_ok {
            return Err(SignatureError::Time(String::from(
                &self.get_certificate_authority(),
            )));
        }
        let pkey = cert
            .get_public_key()
            .map_err(|e| SignatureError::Certificate {
                name: String::from(&self.get_certificate_authority()),
                error: e,
                action: "reading public key".to_string(),
            })?;
        verify(
            pkey.pkey_public().as_ref(),
            &HashableMessage::from(self),
            &self.get_context_hashable(),
            &self.get_signature(),
        )
        .map_err(|e| SignatureError::Certificate {
            name: String::from(&self.get_certificate_authority()),
            error: e,
            action: "verifyinh signature".to_string(),
        })
    }
}

#[derive(Error, Debug)]
pub enum SignatureError {
    #[error(transparent)]
    Keystore(DirectTrustError),
    #[error("Error of certificate {name} during {action}: {error}")]
    Certificate {
        name: String,
        error: OpensslError,
        action: String,
    },
    #[error("Time is not valide for certificate: {0}")]
    Time(String),
}
