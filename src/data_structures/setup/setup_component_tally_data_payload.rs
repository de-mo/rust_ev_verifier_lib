use super::super::deserialize_seq_seq_string_hex_to_seq_seq_bigunit;
use super::super::{
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_data_structure, DataStructureTrait, Signature,
};
use super::encryption_parameters_payload::EncryptionGroup;
use crate::error::{create_verifier_error, VerifierError};
use num::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentTallyDataPayload {
    election_event_id: String,
    verification_card_set_id: String,
    ballot_box_default_title: String,
    encryption_group: EncryptionGroup,
    verification_card_ids: Vec<String>,
    #[serde(deserialize_with = "deserialize_seq_seq_string_hex_to_seq_seq_bigunit")]
    verification_card_public_keys: Vec<Vec<BigUint>>,
    signature: Signature,
}

implement_trait_data_structure!(SetupComponentTallyDataPayload);

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
            .join("verification_card_sets")
            .join("743f2d0fc9fc412798876d7763f78f1b")
            .join("setupComponentTallyDataPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = SetupComponentTallyDataPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
