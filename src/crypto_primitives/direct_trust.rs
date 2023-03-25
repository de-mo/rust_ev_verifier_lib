use openssl::{
    asn1::Asn1Time,
    pkcs12::{ParsedPkcs12_2, Pkcs12},
    pkey::{PKey, Public},
    x509::X509,
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
    Time,
}

impl Display for DirectTrustErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Keystore => "Read Keystore",
            Self::Certificate => "Read Certificate",
            Self::PublicKey => "Public Key",
            Self::Time => "Time Error",
        };
        write!(f, "{s}")
    }
}

pub type DirectTrustError = VerifierError<DirectTrustErrorType>;

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

pub struct SigningCertificate {
    pub authority: CertificateAuthority,
    pub x509: X509,
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
    ) -> Result<SigningCertificate, DirectTrustError> {
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
                    return Ok(SigningCertificate {
                        authority: (*authority).clone(),
                        x509: x.to_owned(),
                    });
                }
            }
        }
        create_result_with_error!(
            DirectTrustErrorType::Certificate,
            format!("Authority {} not found", String::from(authority))
        )
    }
}

impl SigningCertificate {
    pub fn get_public_key(&self) -> Result<PKey<Public>, DirectTrustError> {
        match self.x509.public_key() {
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

    pub fn is_valid_time(&self) -> Result<bool, DirectTrustError> {
        let not_before = self.x509.not_before();
        let not_after = self.x509.not_after();
        let now = match Asn1Time::days_from_now(0) {
            Ok(t) => t,
            Err(e) => return create_result_with_error!(DirectTrustErrorType::Time, "Error now", e),
        };
        Ok(not_before < now && now <= not_after)
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
