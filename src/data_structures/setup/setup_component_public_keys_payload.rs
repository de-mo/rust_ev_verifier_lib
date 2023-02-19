use super::super::deserialize_seq_string_hex_to_seq_bigunit;
use super::super::{SchnorrProofUnderline, Signature};
use super::control_component_public_keys_payload::ControlComponentPublicKeys;
use super::encryption_parameters_payload::EncryptionGroup;
use num::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeys {
    combined_control_component_public_keys: Vec<ControlComponentPublicKeys>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    electoral_board_public_key: Vec<BigUint>,
    electoral_board_schnorr_proofs: Vec<SchnorrProofUnderline>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    election_public_key: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    choice_return_codes_encryption_public_key: Vec<BigUint>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeysPayload {
    encryption_group: EncryptionGroup,
    election_event_id: String,
    setup_component_public_keys: SetupComponentPublicKeys,
    signature: Signature,
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
        let r_eec: Result<SetupComponentPublicKeysPayload, serde_json::Error> =
            serde_json::from_str(&json);
        assert!(r_eec.is_ok())
    }
}
