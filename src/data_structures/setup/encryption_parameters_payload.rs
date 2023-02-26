use super::super::deserialize_string_hex_to_bigunit;
use super::super::{
    implement_trait_fromjson, DataStructureTrait, DeserializeError, DeserializeErrorType, Signature,
};
use crate::error::{create_verifier_error, VerifierError};
use num::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionParametersPayload {
    encryption_group: EncryptionGroup,
    seed: String,
    small_primes: Vec<usize>,
    signature: Signature,
}

implement_trait_fromjson!(EncryptionParametersPayload);

#[derive(Deserialize, Debug)]
pub struct EncryptionGroup {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    p: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    q: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    g: BigUint,
}

#[cfg(test)]
mod test {
    use super::*;
    use num::bigint::ToBigUint;
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
