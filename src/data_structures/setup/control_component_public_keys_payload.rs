use super::super::deserialize_seq_string_hex_to_seq_bigunit;
use super::super::{
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_data_structure, DataStructureTrait, SchnorrProofUnderline, Signature,
};
use super::encryption_parameters_payload::EncryptionGroup;
use crate::error::{create_verifier_error, VerifierError};
use num::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentPublicKeysPayload {
    pub encryption_group: EncryptionGroup,
    pub election_event_id: String,
    pub control_component_public_keys: ControlComponentPublicKeys,
    pub signature: Signature,
}

implement_trait_data_structure!(ControlComponentPublicKeysPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentPublicKeys {
    pub node_id: u8,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub ccrj_choice_return_codes_encryption_public_key: Vec<BigUint>,
    pub ccrj_schnorr_proofs: Vec<SchnorrProofUnderline>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub ccmj_election_public_key: Vec<BigUint>,
    pub ccmj_schnorr_proofs: Vec<SchnorrProofUnderline>,
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
