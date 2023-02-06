use super::super::deserialize_string_hex_to_bigunit;
use num::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct EncryptionGroup {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    p: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    q: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    g: BigUint,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionParametersPayload {
    encryption_group: EncryptionGroup,
    seed: String,
    small_primes: Vec<u32>,
}

#[cfg(test)]
mod test {
    use num::bigint::ToBigUint;

    use super::*;

    #[test]
    fn encryption_group() {
        let json = r#"
        {
            "p": "0xa",
            "q": "0xab",
            "g": "0x2"
        }"#;
        let eg: EncryptionGroup = serde_json::from_str(json).unwrap();
        assert_eq!(eg.p, 10u32.to_biguint().unwrap());
        assert_eq!(eg.q, 171u32.to_biguint().unwrap());
        assert_eq!(eg.g, 2u32.to_biguint().unwrap());
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
            ]
        }"#;
        let eg: EncryptionParametersPayload = serde_json::from_str(json).unwrap();
        assert_eq!(eg.encryption_group.p, 10u32.to_biguint().unwrap());
        assert_eq!(eg.encryption_group.q, 171u32.to_biguint().unwrap());
        assert_eq!(eg.encryption_group.g, 2u32.to_biguint().unwrap());
        assert_eq!(eg.seed, "toto");
        assert_eq!(eg.small_primes, vec![5, 17, 19]);
    }
}
