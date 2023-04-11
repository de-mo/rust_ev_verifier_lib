use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, Proof, Signature},
    deserialize_seq_string_hex_to_seq_bigunit,
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_data_structure, DataStructureTrait,
};
use crate::error::{create_verifier_error, VerifierError};
use num_bigint::BigUint;
use serde::Deserialize;

pub type ControlComponentCodeSharesPayload = Vec<ControlComponentCodeSharesPayloadInner>;

implement_trait_data_structure!(ControlComponentCodeSharesPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeSharesPayloadInner {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub chunk_id: usize,
    pub control_component_code_shares: Vec<ControlComponentCodeShares>,
    pub encryption_group: EncryptionGroup,
    pub node_id: usize,
    pub signature: Signature,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeShares {
    pub verification_card_id: String,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub voter_choice_return_code_generation_public_key: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub voter_vote_cast_return_code_generation_public_key: Vec<BigUint>,
    pub exponentiated_encrypted_partial_choice_return_codes: ExponentiatedEncryptedElement,
    pub encrypted_partial_choice_return_code_exponentiation_proof: Proof,
    pub exponentiated_encrypted_confirmation_key: ExponentiatedEncryptedElement,
    pub encrypted_confirmation_key_exponentiation_proof: Proof,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::file_structure::FileType;
    use std::fs;
    use std::path::Path;

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup")
            .join("verification_card_sets")
            .join("7e8ce00c2c164c268c11cfa7066e3d9f")
            .join("controlComponentCodeSharesPayload.0.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ControlComponentCodeSharesPayload::from_string(&json, &FileType::Json);
        assert!(r_eec.is_ok())
    }
}
