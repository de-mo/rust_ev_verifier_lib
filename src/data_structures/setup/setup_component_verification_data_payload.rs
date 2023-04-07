use super::super::{
    deserialize_seq_string_64_to_seq_bytearray, deserialize_seq_string_hex_to_seq_bigunit,
};
use super::super::{
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_data_structure, DataStructureTrait, ExponentiatedEncryptedElement, Signature,
};
use super::encryption_parameters_payload::EncryptionGroup;
use crate::crypto_primitives::byte_array::ByteArray;
use crate::error::{create_verifier_error, VerifierError};
use num::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentVerificationDataPayload {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    #[serde(deserialize_with = "deserialize_seq_string_64_to_seq_bytearray")]
    pub partial_choice_return_codes_allow_list: Vec<ByteArray>,
    pub chunk_id: usize,
    pub encryption_group: EncryptionGroup,
    pub setup_component_verification_data: Vec<SetupComponentVerificationData>,
    pub combined_correctness_information: CombinedCorrectnessInformation,
    pub signature: Signature,
}

implement_trait_data_structure!(SetupComponentVerificationDataPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentVerificationData {
    pub verification_card_id: String,
    pub encrypted_hashed_squared_confirmation_key: ExponentiatedEncryptedElement,
    pub encrypted_hashed_squared_partial_choice_return_codes: ExponentiatedEncryptedElement,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub verification_card_public_key: Vec<BigUint>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CombinedCorrectnessInformation {
    correctness_information_list: Vec<CorrectnessInformationElt>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CorrectnessInformationElt {
    correctness_id: String,
    number_of_selections: usize,
    number_of_voting_options: usize,
    list_of_write_in_options: Vec<usize>,
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
            .join("setupComponentVerificationDataPayload.0.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = SetupComponentVerificationDataPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}