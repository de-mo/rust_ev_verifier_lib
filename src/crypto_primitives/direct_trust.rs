use openssl::{
    pkcs12::{ParsedPkcs12_2, Pkcs12},
    pkey::{PKey, Public},
    x509::X509Ref,
};
use std::{fmt::Display, fs, path::Path};

use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

const KEYSTORE_FILE_NAME: &str = "public_keys_keystore_verifier.p12";
const PASSWORD_FILE_NAME: &str = "public_keys_keystore_verifier_pw.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectTrustErrorType {
    Keystore,
    Certificate,
    PublicKey,
}

impl Display for DirectTrustErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Keystore => "Read Keystore",
            Self::Certificate => "Read Certificate",
            Self::PublicKey => "Public Key",
        };
        write!(f, "{s}")
    }
}

pub type DirectTrustError = VerifierError<DirectTrustErrorType>;

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

impl From<&CertificateAuthority> for String {
    fn from(value: &CertificateAuthority) -> Self {
        match value {
            CertificateAuthority::Canton => "canton".to_string(),
            CertificateAuthority::SdmConfig => "sdm_config".to_string(),
            CertificateAuthority::SdmTally => "sdm_tally".to_string(),
            CertificateAuthority::VotingServer => "voting_server".to_string(),
            CertificateAuthority::ControlComponent1 => "control_component_1".to_string(),
            CertificateAuthority::ControlComponent2 => "control_component_2".to_string(),
            CertificateAuthority::ControlComponent3 => "control_component_3".to_string(),
            CertificateAuthority::ControlComponent4 => "control_component_4".to_string(),
        }
    }
}

pub struct Keystore {
    pub pcks12: ParsedPkcs12_2,
}

impl Keystore {
    pub fn read_keystore(location: &Path) -> Result<Keystore, DirectTrustError> {
        let bytes = match fs::read(location.join(KEYSTORE_FILE_NAME)) {
            Ok(b) => b,
            Err(e) => {
                return create_result_with_error!(
                    DirectTrustErrorType::Keystore,
                    format!("Error reading keystore file in {:?}", location),
                    e
                )
            }
        };
        let p12: Pkcs12 = match Pkcs12::from_der(&bytes) {
            Ok(p12) => p12,
            Err(e) => {
                return create_result_with_error!(
                    DirectTrustErrorType::Keystore,
                    format!("Error reading content of keystore file in {:?}", location),
                    e
                )
            }
        };
        let pwd = match fs::read_to_string(location.join(PASSWORD_FILE_NAME)) {
            Ok(pwd) => pwd,
            Err(e) => {
                return create_result_with_error!(
                    DirectTrustErrorType::Keystore,
                    format!("Error reading password file in {:?}", location),
                    e
                )
            }
        };
        match p12.parse2(&pwd) {
            Ok(pcks12) => Ok(Keystore { pcks12 }),
            Err(e) => create_result_with_error!(
                DirectTrustErrorType::Keystore,
                format!("Error parsing keystore file in {:?}", location),
                e
            ),
        }
    }

    pub fn get_certificate(
        &self,
        authority: &CertificateAuthority,
    ) -> Result<&X509Ref, DirectTrustError> {
        let cas = match self.pcks12.ca.as_ref() {
            Some(s) => s,
            None => {
                return create_result_with_error!(
                    DirectTrustErrorType::Certificate,
                    "List of CA does not exists"
                )
            }
        };
        for x in cas.iter() {
            for e in x.issuer_name().entries() {
                if e.object().to_string() == "commonName".to_string()
                    && e.data().as_slice() == String::from(authority).as_bytes()
                {
                    return Ok(x);
                }
            }
        }
        create_result_with_error!(
            DirectTrustErrorType::Certificate,
            format!("Authority {} not found", String::from(authority))
        )
    }

    pub fn get_public_key(
        &self,
        authority: &CertificateAuthority,
    ) -> Result<PKey<Public>, DirectTrustError> {
        let x509 = match self.get_certificate(authority) {
            Ok(x509) => x509,
            Err(e) => {
                return create_result_with_error!(
                    DirectTrustErrorType::PublicKey,
                    "Error reading certificate",
                    e
                )
            }
        };
        match x509.public_key() {
            Ok(pk) => Ok(pk),
            Err(e) => {
                return create_result_with_error!(
                    DirectTrustErrorType::PublicKey,
                    "Error reading public key",
                    e
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    fn get_location() -> PathBuf {
        Path::new(".").join("datasets").join("direct-trust")
    }

    #[test]
    fn test_create() {
        let ks = Keystore::read_keystore(&get_location());
        assert!(ks.is_ok())
    }

    #[test]
    fn get_certificate() {
        let ks = Keystore::read_keystore(&get_location()).unwrap();
        let cert = ks.get_certificate(&CertificateAuthority::Canton);
        assert!(cert.is_ok());
        let cert = ks.get_certificate(&CertificateAuthority::SdmConfig);
        assert!(cert.is_ok());
        let cert = ks.get_certificate(&CertificateAuthority::SdmTally);
        assert!(cert.is_ok());
        let cert = ks.get_certificate(&CertificateAuthority::VotingServer);
        assert!(cert.is_ok());
        let cert = ks.get_certificate(&CertificateAuthority::ControlComponent1);
        assert!(cert.is_ok());
        let cert = ks.get_certificate(&CertificateAuthority::ControlComponent2);
        assert!(cert.is_ok());
        let cert = ks.get_certificate(&CertificateAuthority::ControlComponent3);
        assert!(cert.is_ok());
        let cert = ks.get_certificate(&CertificateAuthority::ControlComponent4);
        assert!(cert.is_ok());
    }
}
