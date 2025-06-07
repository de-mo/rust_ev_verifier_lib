// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use crate::data_structures::DataStructureError;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
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

pub struct Keystore(pub(crate) BasisKeystore);

#[derive(Error, Debug)]
#[error(transparent)]
/// Error with DirectTrust
pub struct DirectTrustError(#[from] DirectTrustErrorImpl);

#[derive(Error, Debug)]
enum DirectTrustErrorImpl {
    #[error("Error finding unique file with extension {extension}")]
    FindUniqueFile {
        extension: &'static str,
        source: FindUniqueFileError,
    },
    #[error("Problem reading the keystore {path}")]
    Keystore {
        path: PathBuf,
        source: Box<BasisDirectTrustError>,
    },
    #[error("Error getting the public cartificate for CA {ca}")]
    PublicCertificate {
        ca: String,
        source: Box<BasisDirectTrustError>,
    },
    #[error("Error calculating fingerprint of public certificate for CA {ca}")]
    FingerPrint {
        ca: String,
        source: BasisCryptoError,
    },
}

#[derive(Error, Debug)]
enum FindUniqueFileError {
    #[error("Error reading directory")]
    DirIO {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("No file with extension {extension} found in {path}")]
    FileWithExtNotFound { path: PathBuf, extension: String },
    #[error("More than one file with extension {extension} found in {path}")]
    NotUniqueFile { path: PathBuf, extension: String },
}

#[derive(Error, Debug)]
#[error(transparent)]
/// Error verifiying Signature
pub struct VerifySignatureError(#[from] VerifySignatureErrorImpl);

#[derive(Error, Debug)]
enum VerifySignatureErrorImpl {
    #[error("No certificate authority given")]
    NoCA,
    #[error("Signature error in {msg}")]
    SignatureError {
        msg: String,
        source: Box<SignatureError>,
    },
    #[error("Error getting hashable in {function}")]
    GetHashable {
        function: &'static str,
        source: Box<DataStructureError>,
    },
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
) -> Result<PathBuf, FindUniqueFileError> {
    let pathes = std::fs::read_dir(path)
        .map_err(|e| FindUniqueFileError::DirIO {
            path: path.to_path_buf(),
            source: e,
        })?
        .filter_map(|res| res.ok())
        .map(|f| f.path())
        .filter_map(|path| {
            if path.extension().is_some_and(|ext| ext == extension) {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    match pathes.len() {
        0 => Err(FindUniqueFileError::FileWithExtNotFound {
            path: path.to_path_buf(),
            extension: extension.to_string(),
        }),
        1 => Ok(pathes[0].clone()),
        _ => Err(FindUniqueFileError::NotUniqueFile {
            path: path.to_path_buf(),
            extension: extension.to_string(),
        }),
    }
}

impl TryFrom<&Path> for Keystore {
    type Error = DirectTrustError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let keystore_path = find_unique_file_with_extension(value, "p12").map_err(|e| {
            DirectTrustErrorImpl::FindUniqueFile {
                extension: "p12",
                source: e,
            }
        })?;
        let password_path = find_unique_file_with_extension(value, "txt").map_err(|e| {
            DirectTrustErrorImpl::FindUniqueFile {
                extension: "p12",
                source: e,
            }
        })?;
        Ok(Keystore(
            BasisKeystore::from_pkcs12(&keystore_path, &password_path).map_err(|e| {
                DirectTrustErrorImpl::Keystore {
                    path: keystore_path.clone(),
                    source: Box::new(e),
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
            .map_err(|e| DirectTrustErrorImpl::PublicCertificate {
                ca: ca.to_string(),
                source: Box::new(e),
            })
            .map_err(DirectTrustError::from)?
            .signing_certificate()
            .digest()
            .map_err(|e| DirectTrustErrorImpl::FingerPrint {
                ca: ca.to_string(),
                source: e,
            })
            .map_err(DirectTrustError::from)
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
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError>;

    /// Get the context data of the object according to the specifications
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>>;

    /// Get the Certificate Authority to the specifications
    fn get_certificate_authority(&self) -> Option<CertificateAuthority>;

    /// Get the signature of the object
    fn get_signature(&self) -> ByteArray;

    /// Get the context data of the object according to the context data
    fn get_context_hashable(&'a self) -> HashableMessage<'a> {
        if self.get_context_data().len() == 1 {
            return self.get_context_data()[0].clone();
        }
        HashableMessage::from(self.get_context_data())
    }

    /// Verfiy the signature according to the specifications of Verifier
    fn verifiy_signature(&'a self, keystore: &Keystore) -> Result<bool, VerifySignatureError> {
        let ca = match self.get_certificate_authority() {
            Some(ca) => ca,
            None => return Err(VerifySignatureError::from(VerifySignatureErrorImpl::NoCA)),
        };
        let hashable_message =
            self.get_hashable()
                .map_err(|e| VerifySignatureErrorImpl::GetHashable {
                    function: "verify_signature",
                    source: Box::new(e),
                })?;
        verify_signature(
            &keystore.0,
            &ca.to_string(),
            &hashable_message,
            &self.get_context_hashable(),
            &self.get_signature(),
        )
        .map_err(|e| VerifySignatureErrorImpl::SignatureError {
            msg: "Error verifying the signature".to_string(),
            source: Box::new(e),
        })
        .map_err(VerifySignatureError::from)
    }

    /// Sign according to the specifications of Verifier
    ///
    /// Can be usefull to resign the payload after mocking it
    fn sign(&'a self, keystore: &Keystore) -> Result<ByteArray, VerifySignatureError> {
        let hashable_message =
            self.get_hashable()
                .map_err(|e| VerifySignatureErrorImpl::GetHashable {
                    function: "sign",
                    source: Box::new(e),
                })?;
        sign(&keystore.0, &hashable_message, &self.get_context_hashable())
            .map_err(|e| VerifySignatureErrorImpl::SignatureError {
                msg: "Error signing".to_string(),
                source: Box::new(e),
            })
            .map_err(VerifySignatureError::from)
    }

    /// Verify signatures of an array element
    ///
    /// Per default return an array of one element containing the result of the element verified
    /// The method must be rewritten for a array of elements
    fn verify_signatures(&'a self, keystore: &Keystore) -> Vec<Result<bool, VerifySignatureError>> {
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
