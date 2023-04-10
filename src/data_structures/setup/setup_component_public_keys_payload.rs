use super::{
    super::{
        common_types::{EncryptionGroup, ProofUnderline, Signature},
        deserialize_seq_string_hex_to_seq_bigunit,
        error::{DeserializeError, DeserializeErrorType},
        implement_trait_data_structure, DataStructureTrait,
    },
    control_component_public_keys_payload::ControlComponentPublicKeys,
};
use crate::error::{create_verifier_error, VerifierError};
use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeysPayload {
    pub encryption_group: EncryptionGroup,
    pub election_event_id: String,
    pub setup_component_public_keys: SetupComponentPublicKeys,
    pub signature: Signature,
}

implement_trait_data_structure!(SetupComponentPublicKeysPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeys {
    pub combined_control_component_public_keys: Vec<ControlComponentPublicKeys>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub electoral_board_public_key: Vec<BigUint>,
    pub electoral_board_schnorr_proofs: Vec<ProofUnderline>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub election_public_key: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub choice_return_codes_encryption_public_key: Vec<BigUint>,
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
            .join("setupComponentPublicKeysPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = SetupComponentPublicKeysPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
