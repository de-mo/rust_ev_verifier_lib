use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, Proof, SignatureJson},
    deserialize_seq_string_hex_to_seq_bigunit,
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use crate::error::{create_verifier_error, VerifierError};
use num_bigint::BigUint;
use serde::Deserialize;

pub type ControlComponentCodeSharesPayload = Vec<ControlComponentCodeSharesPayloadInner>;

implement_trait_verifier_data_json_decode!(ControlComponentCodeSharesPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeSharesPayloadInner {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub chunk_id: usize,
    pub control_component_code_shares: Vec<ControlComponentCodeShare>,
    pub encryption_group: EncryptionGroup,
    pub node_id: usize,
    pub signature: SignatureJson,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeShare {
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
    use std::fs;
    use std::path::Path;

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset1-setup-tally")
            .join("setup")
            .join("verification_card_sets")
            .join("681B3488DE4CD4AD7FCED14B7A654169")
            .join("controlComponentCodeSharesPayload.0.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ControlComponentCodeSharesPayload::from_json(&json);
        //println!("{:?}", r_eec);
        assert!(r_eec.is_ok())
    }
}
