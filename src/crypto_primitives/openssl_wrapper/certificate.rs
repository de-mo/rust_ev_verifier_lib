use std::{fs, path::Path};

use openssl::{
    asn1::Asn1Time,
    pkcs12::{ParsedPkcs12_2, Pkcs12},
    pkey::{PKey, Public},
    x509::X509,
};

use super::{OpensslError, OpensslErrorType};
use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

pub struct Keystore {
    pcks12: ParsedPkcs12_2,
}

#[derive(Clone)]
pub struct SigningCertificate {
    authority: String,
    x509: X509,
}

pub type PublicKey = PKey<Public>;

impl Keystore {
    pub fn read_keystore(path: &Path, password: &str) -> Result<Keystore, OpensslError> {
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                return create_result_with_error!(
                    OpensslErrorType::Keystore,
                    format!("Error reading keystore file {:?}", path),
                    e
                )
            }
        };
        let p12: Pkcs12 = match Pkcs12::from_der(&bytes) {
            Ok(p12) => p12,
            Err(e) => {
                return create_result_with_error!(
                    OpensslErrorType::Keystore,
                    format!("Error reading content of keystore file {:?}", path),
                    e
                )
            }
        };
        match p12.parse2(password) {
            Ok(pcks12) => Ok(Keystore { pcks12 }),
            Err(e) => create_result_with_error!(
                OpensslErrorType::Keystore,
                format!("Error parsing keystore file {:?}", path),
                e
            ),
        }
    }

    pub fn get_certificate(&self, authority: &str) -> Result<SigningCertificate, OpensslError> {
        let cas = match self.pcks12.ca.as_ref() {
            Some(s) => s,
            None => {
                return create_result_with_error!(
                    OpensslErrorType::Certificate,
                    "List of CA does not exists"
                )
            }
        };
        for x in cas.iter() {
            for e in x.issuer_name().entries() {
                if e.object().to_string() == "commonName".to_string()
                    && e.data().as_slice() == authority.as_bytes()
                {
                    return Ok(SigningCertificate {
                        authority: authority.to_owned(),
                        x509: x.to_owned(),
                    });
                }
            }
        }
        create_result_with_error!(
            OpensslErrorType::Certificate,
            format!("Authority {} not found", authority)
        )
    }
}

impl SigningCertificate {
    pub fn get_public_key(&self) -> Result<PublicKey, OpensslError> {
        match self.x509.public_key() {
            Ok(pk) => Ok(pk),
            Err(e) => {
                return create_result_with_error!(
                    OpensslErrorType::PublicKey,
                    "Error reading public key",
                    e
                )
            }
        }
    }

    pub fn authority(&self) -> &str {
        &self.authority
    }

    pub fn is_valid_time(&self) -> Result<bool, OpensslError> {
        let not_before = self.x509.not_before();
        let not_after = self.x509.not_after();
        let now = match Asn1Time::days_from_now(0) {
            Ok(t) => t,
            Err(e) => return create_result_with_error!(OpensslErrorType::Time, "Error now", e),
        };
        Ok(not_before < now && now <= not_after)
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    const PASSWORD: &str = "testPassword";

    fn get_file() -> PathBuf {
        Path::new(".")
            .join("datasets")
            .join("direct-trust")
            .join("public_keys_keystore_verifier.p12")
    }

    #[test]
    fn test_create() {
        let ks = Keystore::read_keystore(&get_file(), PASSWORD);
        assert!(ks.is_ok());
        let ks_err = Keystore::read_keystore(&get_file(), "toto");
        assert!(ks_err.is_err());
        let ks_err2 = Keystore::read_keystore(Path::new("./toto.p12"), PASSWORD);
        assert!(ks_err2.is_err());
    }

    #[test]
    fn get_certificate() {
        let ks = Keystore::read_keystore(&get_file(), PASSWORD).unwrap();
        let cert = ks.get_certificate("canton");
        assert!(cert.is_ok());
        assert_eq!(cert.unwrap().authority(), "canton");
        let cert = ks.get_certificate("sdm_config");
        assert!(cert.is_ok());
        let cert = ks.get_certificate("sdm_tally");
        assert!(cert.is_ok());
        let cert = ks.get_certificate("voting_server");
        assert!(cert.is_ok());
        let cert = ks.get_certificate("control_component_1");
        assert!(cert.is_ok());
        let cert = ks.get_certificate("control_component_2");
        assert!(cert.is_ok());
        let cert = ks.get_certificate("control_component_3");
        assert!(cert.is_ok());
        let cert = ks.get_certificate("control_component_4");
        assert!(cert.is_ok());
        let cert = ks.get_certificate("toto");
        assert!(cert.is_err());
    }
}
