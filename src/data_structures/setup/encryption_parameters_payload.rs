use super::super::{
    common_types::{SignatureJson, EncryptionParametersDef},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
    CheckDomainTrait
};
use anyhow::anyhow;
use rust_ev_crypto_primitives::{EncryptionParameters,
    ByteArray,  HashableMessage,
    VerifiySignatureTrait,
};
use serde::Deserialize;
use crate::config::Config as VerifierConfig;
use crate::direct_trust::CertificateAuthority;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionParametersPayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub seed: String,
    pub small_primes: Vec<usize>,
    pub signature: SignatureJson,
}

implement_trait_verifier_data_json_decode!(EncryptionParametersPayload);

impl CheckDomainTrait for EncryptionParametersPayload {
    fn check_domain(&self) -> Vec<anyhow::Error> {
        let mut res = vec![];
        res.append(&mut self.encryption_group.check_domain());
        // For 5.02
        if !self.small_primes.len() == VerifierConfig::maximum_number_of_voting_options() {
            res.push(
                anyhow!(
                    format!("The list of small primes {} is not equal to the maximal number of voting options {}", 
                        self.small_primes.len(), 
                        VerifierConfig::maximum_number_of_voting_options()
                    )
                )
            )
        }
        // for 5.02
        let mut sp = self.small_primes.clone();
        sp.sort();
        if sp != self.small_primes {
            res.push(anyhow!("Small primes list is not in ascending order"))
        }
        // for 5.02
        if sp[0] < 5 {
            res.push(anyhow!("The small primes contain 2 or 3, what is not allowed"))
        }
        res
    }
}

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
    type Error=std::convert::Infallible;

    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Self::Error> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("encryption parameters")]
    }

    fn get_certificate_authority(&self) -> Result<String, Self::Error> {
        Ok(String::from(&CertificateAuthority::SdmConfig))
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
        let mut de = serde_json::Deserializer::from_str(json);
        let eg = EncryptionParametersDef::deserialize(&mut de).unwrap();
        assert_eq!(eg.p(), &10usize.to_biguint().unwrap());
        assert_eq!(eg.q(), &171usize.to_biguint().unwrap());
        assert_eq!(eg.g(), &2usize.to_biguint().unwrap());
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
        assert_eq!(eg.encryption_group.p(), &10usize.to_biguint().unwrap());
        assert_eq!(eg.encryption_group.q(), &171usize.to_biguint().unwrap());
        assert_eq!(eg.encryption_group.g(), &2usize.to_biguint().unwrap());
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
        assert!(r_eg.is_ok())
    }

    #[test]
    fn test_check_domain() {
        let path = test_dataset_tally_path()
            .join("setup")
            .join("encryptionParametersPayload.json");
        let json = fs::read_to_string(path).unwrap();
        let eg = EncryptionParametersPayload::from_json(&json).unwrap();
        assert!(eg.check_domain().is_empty())
    }
}
