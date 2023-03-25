use std::fmt::Display;

use openssl::{
    asn1::Asn1Time,
    pkcs12::{ParsedPkcs12_2, Pkcs12},
    pkey::{PKey, Public},
    x509::X509,
};

use super::direct_trust::{CertificateAuthority, Keystore};
use super::{byte_array::ByteArray, hashing::RecursiveHashable};
use crate::data_structures::SignatureTrait;
use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

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

fn verify(pkey: &PKey<Public>, bytes: &ByteArray, signature: &ByteArray) -> bool {
    todo!()
}

fn verifiy_signature(
    keystore: &Keystore,
    authority_id: &CertificateAuthority,
    hashable: &RecursiveHashable,
    context_data: &RecursiveHashable,
    signature: &ByteArray,
) -> Result<bool, SignatureError> {
    let cert = match keystore.get_certificate(authority_id) {
        Ok(c) => c,
        Err(e) => {
            return create_result_with_error!(
                SignatureErrorType::DirectTrust,
                "Error reading PK",
                e
            )
        }
    };
    let time_ok = match cert.is_valid_time() {
        Ok(b) => b,
        Err(e) => {
            return create_result_with_error!(
                SignatureErrorType::DirectTrust,
                "Error testing time",
                e
            )
        }
    };
    if !time_ok {
        return create_result_with_error!(SignatureErrorType::Validation, "Time is not valide");
    }
    let pkey = match cert.get_public_key() {
        Ok(pk) => pk,
        Err(e) => {
            return create_result_with_error!(
                SignatureErrorType::DirectTrust,
                "Error reading PK",
                e
            )
        }
    };
    let h = RecursiveHashable::Composite(vec![hashable.to_owned(), context_data.to_owned()])
        .recursive_hash();
    Ok(verify(&pkey, &h, signature))
}

pub trait VerifiySignatureTrait<'a>
where
    Self: 'a + SignatureTrait,
    RecursiveHashable: From<&'a Self>,
{
    fn get_context_data(&self) -> RecursiveHashable;
    fn get_certificate_authority(&self) -> CertificateAuthority;
    fn verifiy_signature(&'a self, keystore: &Keystore) -> Result<bool, SignatureError> {
        verifiy_signature(
            keystore,
            &self.get_certificate_authority(),
            &RecursiveHashable::from(self),
            &self.get_context_data(),
            &self.get_signature(),
        )
    }
}
