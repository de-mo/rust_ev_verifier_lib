//! Module to implement the signature and the verification of the signature of
//! an object.

use anyhow::{anyhow, bail};

use super::{byte_array::ByteArray, hashing::HashableMessage};
use super::{
    direct_trust::{CertificateAuthority, DirectTrust},
    openssl_wrapper::verify,
};
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
    fn verifiy_signature(&'a self, location: &Path) -> anyhow::Result<bool> {
        let dt = DirectTrust::new(location, &self.get_certificate_authority()).map_err(|e| {
            anyhow!(e).context(format!(
                "Error reading keystore {}",
                location.to_str().unwrap()
            ))
        })?;
        let cert = dt.signing_certificate();
        let time_ok = cert.is_valid_time().map_err(|e| {
            anyhow!(e).context(format!(
                "Error testing time for certificate {}",
                String::from(&self.get_certificate_authority())
            ))
        })?;
        if !time_ok {
            bail!(format!(
                "Time is not valide for certificate {}",
                String::from(&self.get_certificate_authority())
            ))
        }
        let pkey = cert.get_public_key().map_err(|e| {
            anyhow!(e).context(format!(
                "Error reading public key for certificate {}",
                String::from(&self.get_certificate_authority())
            ))
        })?;
        verify(
            pkey.as_ref(),
            &HashableMessage::from(self),
            &self.get_context_hashable(),
            &self.get_signature(),
        )
        .map_err(|e| anyhow!(e).context(format!("Error during verification ofsignature")))
    }
}
