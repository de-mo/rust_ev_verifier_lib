use super::super::{
    common_types::{EncryptionGroup, Signature, SignatureTrait},
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_data_structure, DataStructureTrait,
};
use crate::{
    crypto_primitives::{
        direct_trust::CertificateAuthority, hashing::RecursiveHashable,
        signature::VerifiySignatureTrait,
    },
    error::{create_verifier_error, VerifierError},
};
use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionParametersPayload {
    pub encryption_group: EncryptionGroup,
    pub seed: String,
    pub small_primes: Vec<usize>,
    pub signature: Signature,
}

implement_trait_data_structure!(EncryptionParametersPayload);

impl<'a> From<&EncryptionParametersPayload> for RecursiveHashable {
    fn from(value: &EncryptionParametersPayload) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.encryption_group.p));
        elts.push(Self::from(&value.encryption_group.q));
        elts.push(Self::from(&value.encryption_group.g));
        elts.push(Self::from(&value.seed));
        let sp_hash: Vec<BigUint> = value
            .small_primes
            .iter()
            .map(|p| BigUint::from(*p))
            .collect();
        elts.push(Self::from(&sp_hash));
        Self::from(&elts)
    }
}

impl VerifiySignatureTrait<'_> for EncryptionParametersPayload {
    fn get_context_data(&self) -> RecursiveHashable {
        RecursiveHashable::from(&"encryption parameters".to_string())
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::SdmConfig
    }
}

impl SignatureTrait for EncryptionParametersPayload {
    fn get_signature_struct(&self) -> &Signature {
        &self.signature
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use num_bigint::ToBigUint;
    use std::fs;
    use std::path::Path;

    #[test]
    fn encryption_group() {
        let json = r#"
        {
            "p": "0xa",
            "q": "0xab",
            "g": "0x2"
        }"#;
        let eg: EncryptionGroup = serde_json::from_str(json).unwrap();
        assert_eq!(eg.p, 10usize.to_biguint().unwrap());
        assert_eq!(eg.q, 171usize.to_biguint().unwrap());
        assert_eq!(eg.g, 2usize.to_biguint().unwrap());
    }

    #[test]
    fn encryption_parameters_payload() {
        let json = r#"
        {
            "encryptionGroup": {
                "p": "0xa",
                "q": "0xab",
                "g": "0x2"
            },
            "seed": "toto",
            "smallPrimes": [
                5,
                17,
                19
            ],
            "signature": {
		        "signatureContents": "fifi"
	        }
        }
        
        "#;
        let eg: EncryptionParametersPayload = serde_json::from_str(json).unwrap();
        assert_eq!(eg.encryption_group.p, 10usize.to_biguint().unwrap());
        assert_eq!(eg.encryption_group.q, 171usize.to_biguint().unwrap());
        assert_eq!(eg.encryption_group.g, 2usize.to_biguint().unwrap());
        assert_eq!(eg.seed, "toto");
        assert_eq!(eg.small_primes, vec![5, 17, 19]);
        assert_eq!(eg.signature.signature_contents, "fifi")
    }

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup")
            .join("encryptionParametersPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eg = EncryptionParametersPayload::from_json(&json);
        //let r_eg: Result<EncryptionParametersPayload, serde_json::Error> =
        //    serde_json::from_str(&json);
        assert!(r_eg.is_ok())
    }
}
