use super::super::{
    common_types::{EncryptionGroup, SignatureJson},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use anyhow::anyhow;
use rust_ev_crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionParametersPayload {
    pub encryption_group: EncryptionGroup,
    pub seed: String,
    pub small_primes: Vec<usize>,
    pub signature: SignatureJson,
}

implement_trait_verifier_data_json_decode!(EncryptionParametersPayload);

impl<'a> From<&'a EncryptionParametersPayload> for HashableMessage<'a> {
    fn from(value: &'a EncryptionParametersPayload) -> Self {
        let mut elts = vec![Self::from(&value.encryption_group), Self::from(&value.seed)];
        let sp_hash: Vec<HashableMessage> = value
            .small_primes
            .iter()
            .map(HashableMessage::from)
            .collect();
        elts.push(Self::from(sp_hash));
        Self::from(elts)
    }
}

impl<'a> VerifiySignatureTrait<'a> for EncryptionParametersPayload {
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("encryption parameters")]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::SdmConfig
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_dataset_tally_path;
    use num_bigint::ToBigUint;
    use std::fs;

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
        let path = test_dataset_tally_path()
            .join("setup")
            .join("encryptionParametersPayload.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eg = EncryptionParametersPayload::from_json(&json);
        //let r_eg: Result<EncryptionParametersPayload, serde_json::Error> =
        //    serde_json::from_str(&json);
        assert!(r_eg.is_ok())
    }
}
