use super::super::deserialize_seq_string_hex_to_seq_bigunit;
use super::super::{ExponentiatedEncryptedElement, SchnorrProof, Signature};
use super::encryption_parameters_payload::EncryptionGroup;
use num::BigUint;
use serde::Deserialize;

pub type ControlComponentCodeSharesPayload = Vec<ControlComponentCodeSharesPayloadInner>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeSharesPayloadInner {
    election_event_id: String,
    verification_card_set_id: String,
    chunk_id: usize,
    control_component_code_shares: Vec<ControlComponentCodeShares>,
    encryption_group: EncryptionGroup,
    node_id: usize,
    signature: Signature,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeShares {
    verification_card_id: String,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    voter_choice_return_code_generation_public_key: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    voter_vote_cast_return_code_generation_public_key: Vec<BigUint>,
    exponentiated_encrypted_partial_choice_return_codes: ExponentiatedEncryptedElement,
    encrypted_partial_choice_return_code_exponentiation_proof: SchnorrProof,
    exponentiated_encrypted_confirmation_key: ExponentiatedEncryptedElement,
    encrypted_confirmation_key_exponentiation_proof: SchnorrProof,
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
            .join("verification_card_sets")
            .join("743f2d0fc9fc412798876d7763f78f1b")
            .join("controlComponentCodeSharesPayload.0.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec: Result<ControlComponentCodeSharesPayload, serde_json::Error> =
            serde_json::from_str(&json);
        assert!(r_eec.is_ok())
    }
}
