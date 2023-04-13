//! Module implementing the direct trust

use super::openssl_wrapper::certificate::{Keystore, SigningCertificate};
use crate::error::{create_verifier_error, VerifierError};
use std::{fmt::Display, fs, path::Path};

const KEYSTORE_FILE_NAME: &str = "public_keys_keystore_verifier.p12";
const PASSWORD_FILE_NAME: &str = "public_keys_keystore_verifier_pw.txt";

/// List of valide Certificate authorities
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Struct representing a direct trust certificate
#[derive(Clone)]
pub struct DirectTrust {
    authority: CertificateAuthority,
    cert: SigningCertificate,
}

impl DirectTrust {
    /// Create a new direct trust certificate reading the store at location for
    /// the given authority
    pub fn new(
        location: &Path,
        authority: &CertificateAuthority,
    ) -> Result<DirectTrust, DirectTrustError> {
        let file = location.join(KEYSTORE_FILE_NAME);
        let file_pwd = location.join(PASSWORD_FILE_NAME);
        let pwd = fs::read_to_string(&file_pwd).map_err(|e| {
            create_verifier_error!(
                DirectTrustErrorType::Password,
                format!("Error reading password file {}", &file_pwd.display()),
                e
            )
        })?;
        let ks = Keystore::read_keystore(&file, &pwd).map_err(|e| {
            create_verifier_error!(
                DirectTrustErrorType::Error,
                format!("Error reading keystore {}", file.display()),
                e
            )
        })?;
        let cert = ks.get_certificate(&String::from(authority)).map_err(|e| {
            create_verifier_error!(
                DirectTrustErrorType::Error,
                format!("Error reading certificate {}", String::from(authority)),
                e
            )
        })?;
        Ok(DirectTrust {
            authority: authority.clone(),
            cert: cert.to_owned(),
        })
    }

    /// Get authority of the certificate
    pub fn authority(&self) -> &CertificateAuthority {
        &self.authority
    }

    /// Get the certificate of the authority
    pub fn signing_certificate(&self) -> &SigningCertificate {
        &self.cert
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectTrustErrorType {
    Error,
    Password,
}

impl Display for DirectTrustErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Error => "General Error",
            Self::Password => "Password Error",
        };
        write!(f, "{s}")
    }
}

pub type DirectTrustError = VerifierError<DirectTrustErrorType>;

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

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    fn get_location() -> PathBuf {
        Path::new(".").join("datasets").join("direct-trust")
    }

    #[test]
    fn test_create() {
        let dt = DirectTrust::new(&get_location(), &CertificateAuthority::Canton);
        assert!(dt.is_ok());
        let dt = DirectTrust::new(&get_location(), &CertificateAuthority::SdmConfig);
        assert!(dt.is_ok());
        let dt = DirectTrust::new(&get_location(), &CertificateAuthority::SdmTally);
        assert!(dt.is_ok());
        let dt = DirectTrust::new(&get_location(), &CertificateAuthority::VotingServer);
        assert!(dt.is_ok());
        let dt = DirectTrust::new(&get_location(), &CertificateAuthority::ControlComponent1);
        assert!(dt.is_ok());
        let dt = DirectTrust::new(&get_location(), &CertificateAuthority::ControlComponent2);
        assert!(dt.is_ok());
        let dt = DirectTrust::new(&get_location(), &CertificateAuthority::ControlComponent3);
        assert!(dt.is_ok());
        let dt = DirectTrust::new(&get_location(), &CertificateAuthority::ControlComponent4);
        assert!(dt.is_ok());
        let dt_err = DirectTrust::new(Path::new("./toto"), &CertificateAuthority::Canton);
        assert!(dt_err.is_err());
    }
}
