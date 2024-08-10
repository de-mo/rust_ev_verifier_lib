use std::{
    fmt::Display, path::{Path, PathBuf}, slice::Iter
};

use anyhow::{anyhow, Context};
use rust_ev_crypto_primitives::{
    sign, verify_signature, ByteArray, HashableMessage, Keystore as BasisKeystore,
};

pub struct Keystore(pub BasisKeystore);

/// List of valide Certificate authorities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateAuthority {
    Canton,
    SdmConfig,
    SdmTally,
    VotingServer,
    ControlComponent1,
    ControlComponent2,
    ControlComponent3,
    ControlComponent4,
}

impl CertificateAuthority {
    pub fn get_ca_cc(node: &usize) -> Option<Self> {
        match node {
            1 => Some(Self::ControlComponent1),
            2 => Some(Self::ControlComponent2),
            3 => Some(Self::ControlComponent3),
            4 => Some(Self::ControlComponent4),
            _ => None,
        }
    }

    pub fn iter() -> Iter<'static, CertificateAuthority> {
        static AUTHORITIES: [CertificateAuthority; 8] = [
            CertificateAuthority::Canton,
            CertificateAuthority::SdmConfig,
            CertificateAuthority::SdmTally,
            CertificateAuthority::VotingServer,
            CertificateAuthority::ControlComponent1,
            CertificateAuthority::ControlComponent2,
            CertificateAuthority::ControlComponent3,
            CertificateAuthority::ControlComponent4,
        ];
        AUTHORITIES.iter()
    }
}

impl Display for CertificateAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",match self {
            CertificateAuthority::Canton => "canton",
            CertificateAuthority::SdmConfig => "sdm_config",
            CertificateAuthority::SdmTally => "sdm_tally",
            CertificateAuthority::VotingServer => "voting_server",
            CertificateAuthority::ControlComponent1 => "control_component_1",
            CertificateAuthority::ControlComponent2 => "control_component_2",
            CertificateAuthority::ControlComponent3 => "control_component_3",
            CertificateAuthority::ControlComponent4 => "control_component_4",
        } )
    }
}

pub fn find_unique_file_with_extension(path: &Path, extension: &str) -> anyhow::Result<PathBuf> {
    let pathes = std::fs::read_dir(path)
        .map_err(|e| anyhow!(e))?
        .filter_map(|res| res.ok())
        .map(|f| f.path())
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext == extension) {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if pathes.len() != 1 {
        return Err(anyhow!("Too many files or no file found"));
    }
    Ok(pathes[0].clone())
}

impl TryFrom<&Path> for Keystore {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let keystore_path =
            find_unique_file_with_extension(value, "p12").context("Error reading keystore path")?;
        let password_path =
            find_unique_file_with_extension(value, "txt").context("Error reading password path")?;
        Ok(Keystore(
            BasisKeystore::from_pkcs12(&keystore_path, &password_path)
                .context("Problem reading the keystore")?,
        ))
    }
}

/// Trait that must be implemented for each object implementing a signature to be verified
///
/// The following function are to be implemented for the object to make it running:
/// - [VerifiySignatureTrait::get_hashable] Get the [HashableMessage] for the object
/// - [VerifiySignatureTrait::get_context_data] Get the context data as [HashableMessage] for the object, according to the specifications
/// - [VerifiySignatureTrait::get_certificate_authority] Certificate Authority of the certificate to fin the certificate in the keystore
/// - [VerifiySignatureTrait::get_signature] Get the signature of the object
pub trait VerifiySignatureTrait<'a>
where
    Self: 'a,
{
    /// Get the hashable from the object
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>>;

    /// Get the context data of the object according to the specifications
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>>;

    /// Get the Certificate Authority to the specifications
    fn get_certificate_authority(&self) -> Option<CertificateAuthority>;

    /// Get the signature of the object
    fn get_signature(&self) -> ByteArray;

    /// Get the context data of the object according to the context data
    fn get_context_hashable(&'a self) -> HashableMessage {
        if self.get_context_data().len() == 1 {
            return self.get_context_data()[0].clone();
        }
        HashableMessage::from(self.get_context_data())
    }

    /// Verfiy the signature according to the specifications of Verifier
    fn verifiy_signature(&'a self, keystore: &Keystore) -> anyhow::Result<bool> {
        let ca = match self.get_certificate_authority() {
            Some(ca) => ca,
            None => return Err(anyhow!("Error getting the certificate")),
        };
        let hashable_message = self
            .get_hashable()
            .context("Error getting the hashable message")?;
        verify_signature(
            &keystore.0,
            &ca.to_string(),
            &hashable_message,
            &self.get_context_hashable(),
            &self.get_signature(),
        )
        .context("Error verifying the signature")
    }

    /// Sign according to the specifications of Verifier
    ///
    /// Can be usefull to resign the payload after mocking it
    fn sign(&'a self, keystore: &Keystore) -> anyhow::Result<ByteArray> {
        let hashable_message = self
            .get_hashable()
            .context("Error getting the hashable message")?;
        sign(&keystore.0, &hashable_message, &self.get_context_hashable()).context("Error signing")
    }

    /// Verify signatures of an array element
    ///
    /// Per default return an array of one element containing the result of the element verified
    /// The method must be rewritten for a array of elements
    fn verify_signatures(&'a self, keystore: &Keystore) -> Vec<anyhow::Result<bool>> {
        vec![self.verifiy_signature(keystore)]
    }
}

#[cfg(test)]
mod test {
    use crate::config::test::CONFIG_TEST;

    use super::*;

    #[test]
    fn test_create() {
        let dt = CONFIG_TEST.keystore().unwrap();
        assert!(dt
            .0
            .public_certificate(&CertificateAuthority::Canton.to_string())
            .is_ok());
        assert!(dt
            .0
            .public_certificate(&CertificateAuthority::SdmConfig.to_string())
            .is_ok());
        assert!(dt
            .0
            .public_certificate(&CertificateAuthority::SdmTally.to_string())
            .is_ok());
        assert!(dt
            .0
            .public_certificate(&CertificateAuthority::VotingServer.to_string())
            .is_ok());
        assert!(dt
            .0
            .public_certificate(&CertificateAuthority::ControlComponent1.to_string())
            .is_ok());
        assert!(dt
            .0
            .public_certificate(&CertificateAuthority::ControlComponent2.to_string())
            .is_ok());
        assert!(dt
            .0
            .public_certificate(&CertificateAuthority::ControlComponent3.to_string())
            .is_ok());
        assert!(dt
            .0
            .public_certificate(&CertificateAuthority::ControlComponent4.to_string())
            .is_ok());
    }
}
