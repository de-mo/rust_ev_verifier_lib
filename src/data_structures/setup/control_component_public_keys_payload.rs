use super::super::deserialize_seq_string_hex_to_seq_bigunit;
use super::super::{
    implement_trait_fromjson, DeserializeError, DeserializeErrorType, FromJson,
    SchnorrProofUnderline, Signature,
};
use super::encryption_parameters_payload::EncryptionGroup;
use crate::error::{create_verifier_error, VerifierError};
use num::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentPublicKeysPayload {
    encryption_group: EncryptionGroup,
    election_event_id: String,
    control_component_public_keys: ControlComponentPublicKeys,
    signature: Signature,
}

implement_trait_fromjson!(ControlComponentPublicKeysPayload);

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentPublicKeys {
    node_id: u8,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    ccrj_choice_return_codes_encryption_public_key: Vec<BigUint>,
    ccrj_schnorr_proofs: Vec<SchnorrProofUnderline>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    ccmj_election_public_key: Vec<BigUint>,
    ccmj_schnorr_proofs: Vec<SchnorrProofUnderline>,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup")
            .join("controlComponentPublicKeysPayload.1.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ControlComponentPublicKeysPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
