use rust_ev_crypto_primitives::{
    basic_crypto_functions::BasisCryptoError,
    direct_trust::{DirectTrustError as BasisDirectTrustError, Keystore as BasisKeystore},
    signature::{sign, verify_signature, SignatureError},
    ByteArray, HashableMessage,
};
use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
    slice::Iter,
};
use thiserror::Error;

use crate::data_structures::XMLError;

pub struct Keystore(pub(crate) BasisKeystore);

// Enum representing the direct trust errors
#[derive(Error, Debug)]
pub enum DirectTrustError {
    #[error("IO error {msg} -> caused by: {source}")]
    IO { msg: String, source: std::io::Error },
    #[error("No file with extension {0} found")]
    FileNotFound(String),
    #[error("More than one file with extension {0} found")]
    NotUniqueFile(String),
    #[error("Keystore error {msg} -> caused by: {source}")]
    Keystore {
        msg: String,
        source: BasisDirectTrustError,
    },
    #[error("Crypto error {msg} -> caused by: {source}")]
    Crypto {
        msg: String,
        source: BasisCryptoError,
    },
    #[error("No signing Keystore for Voting Server")]
    NoSigningVotingServer,
}

// Enum representing the direct trust errors
#[derive(Error, Debug)]
pub enum VerifySignatureError {
    #[error("No certificate authority given")]
    NoCA,
    #[error("Signature error {msg} -> caused by: {source}")]
    SignatureError { msg: String, source: SignatureError },
    #[error("XML error {msg} -> caused by: {source}")]
    XMLError { msg: String, source: XMLError },
}

/// List of valide Certificate authorities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CertificateAuthority {
    Canton,
    SdmConfig,
    SdmTally,
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
        static AUTHORITIES: [CertificateAuthority; 7] = [
            CertificateAuthority::Canton,
            CertificateAuthority::SdmConfig,
            CertificateAuthority::SdmTally,
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
        write!(
            f,
            "{}",
            match self {
                CertificateAuthority::Canton => "canton",
                CertificateAuthority::SdmConfig => "sdm_config",
                CertificateAuthority::SdmTally => "sdm_tally",
                CertificateAuthority::ControlComponent1 => "control_component_1",
                CertificateAuthority::ControlComponent2 => "control_component_2",
                CertificateAuthority::ControlComponent3 => "control_component_3",
                CertificateAuthority::ControlComponent4 => "control_component_4",
            }
        )
    }
}

fn find_unique_file_with_extension(
    path: &Path,
    extension: &str,
) -> Result<PathBuf, DirectTrustError> {
    let pathes = std::fs::read_dir(path)
        .map_err(|e| DirectTrustError::IO {
            msg: path.as_os_str().to_str().unwrap().to_string(),
            source: e,
        })?
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
    match pathes.len() {
        0 => Err(DirectTrustError::FileNotFound(extension.to_string())),
        1 => Ok(pathes[0].clone()),
        _ => Err(DirectTrustError::NotUniqueFile(extension.to_string())),
    }
}

impl TryFrom<&Path> for Keystore {
    type Error = DirectTrustError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let keystore_path = find_unique_file_with_extension(value, "p12")?;
        let password_path = find_unique_file_with_extension(value, "txt")?;
        Ok(Keystore(
            BasisKeystore::from_pkcs12(&keystore_path, &password_path).map_err(|e| {
                DirectTrustError::Keystore {
                    msg: format!(
                        "Problem reading the keystore in {}",
                        value.as_os_str().to_str().unwrap()
                    ),
                    source: e,
                }
            })?,
        ))
    }
}

impl Keystore {
    pub fn fingerprints(
        &self,
    ) -> Result<HashMap<CertificateAuthority, ByteArray>, DirectTrustError> {
        let mut res = HashMap::new();
        for ca in CertificateAuthority::iter() {
            res.insert(*ca, self.fingerprint(*ca)?);
        }
        Ok(res)
    }

    pub fn fingerprint(&self, ca: CertificateAuthority) -> Result<ByteArray, DirectTrustError> {
        self.0
            .public_certificate(&ca.to_string())
            .map_err(|e| DirectTrustError::Keystore {
                msg: "calculating fingerprint".to_string(),
                source: e,
            })?
            .signing_certificate()
            .digest()
            .map_err(|e| DirectTrustError::Crypto {
                msg: "calculating fingerprint".to_string(),
                source: e,
            })
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
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Box<VerifySignatureError>>;

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
    fn verifiy_signature(&'a self, keystore: &Keystore) -> Result<bool, Box<VerifySignatureError>> {
        let ca = match self.get_certificate_authority() {
            Some(ca) => ca,
            None => return Err(Box::new(VerifySignatureError::NoCA)),
        };
        let hashable_message = self.get_hashable()?;
        verify_signature(
            &keystore.0,
            &ca.to_string(),
            &hashable_message,
            &self.get_context_hashable(),
            &self.get_signature(),
        )
        .map_err(|e| {
            Box::new(VerifySignatureError::SignatureError {
                msg: "Error verifying the signature".to_string(),
                source: e,
            })
        })
    }

    /// Sign according to the specifications of Verifier
    ///
    /// Can be usefull to resign the payload after mocking it
    fn sign(&'a self, keystore: &Keystore) -> Result<ByteArray, Box<VerifySignatureError>> {
        let hashable_message = self.get_hashable()?;
        sign(&keystore.0, &hashable_message, &self.get_context_hashable()).map_err(|e| {
            Box::new(VerifySignatureError::SignatureError {
                msg: "Error signing".to_string(),
                source: e,
            })
        })
    }

    /// Verify signatures of an array element
    ///
    /// Per default return an array of one element containing the result of the element verified
    /// The method must be rewritten for a array of elements
    fn verify_signatures(
        &'a self,
        keystore: &Keystore,
    ) -> Vec<Result<bool, Box<VerifySignatureError>>> {
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
