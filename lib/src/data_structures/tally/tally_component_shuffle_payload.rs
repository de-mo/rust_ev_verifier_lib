use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, SignatureJson},
    deserialize_seq_string_hex_to_seq_bigunit, deserialize_string_hex_to_bigunit,
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use crate::data_structures::common_types::DecryptionProof;
use anyhow::anyhow;
use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TallyComponentShufflePayload {
    pub(crate) encryption_group: EncryptionGroup,
    pub(crate) election_event_id: String,
    pub(crate) ballot_box_id: String,
    pub(crate) verifiable_shuffle: VerifiableShuffle,
    pub(crate) verifiable_plaintext_decryption: VerifiablePlaintextDecryption,
    pub(crate) signature: SignatureJson,
}
implement_trait_verifier_data_json_decode!(TallyComponentShufflePayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VerifiableShuffle {
    pub(crate) shuffled_ciphertexts: Vec<ExponentiatedEncryptedElement>,
    pub(crate) shuffle_argument: ShuffleArgument,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VerifiablePlaintextDecryption {
    pub(crate) decrypted_votes: Vec<DecryptedVote>,
    pub(crate) decryption_proofs: Vec<DecryptionProof>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ShuffleArgument {
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    #[serde(rename = "c_A")]
    pub(crate) c_a: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    #[serde(rename = "c_B")]
    pub(crate) c_b: Vec<BigUint>,
    #[serde(rename = "productArgument")]
    pub(crate) product_argument: ProductArgument,
    #[serde(rename = "multiExponentiationArgument")]
    pub(crate) multi_exponentiation_argument: MultiExponentiationArgument,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductArgument {
    pub(crate) single_value_product_argument: SingleValueProductArgument,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SingleValueProductArgument {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) c_d: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) c_delta: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "c_Delta")]
    pub(crate) c_delta_upper: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub(crate) a_tilde: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub(crate) b_tilde: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) r_tilde: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) s_tilde: BigUint,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultiExponentiationArgument {
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    #[serde(rename = "c_A_0")]
    pub(crate) c_a_0: BigUint,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    #[serde(rename = "c_B")]
    pub(crate) c_b: Vec<BigUint>,
    #[serde(rename = "E")]
    pub(crate) e: Vec<ExponentiatedEncryptedElement>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub(crate) a: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) r: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) b: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) s: BigUint,
    #[serde(deserialize_with = "deserialize_string_hex_to_bigunit")]
    pub(crate) tau: BigUint,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DecryptedVote {
    pub(crate) message: Vec<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::test::dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = dataset_tally_path()
            .join("tally")
            .join("ballot_boxes")
            .join("4AB4F95B8114C1DFEDB9586ADBFE36B3")
            .join("tallyComponentShufflePayload.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = TallyComponentShufflePayload::from_json(&json);
        println!("{:?}", r_eec.as_ref().err());
        assert!(r_eec.is_ok())
    }
}
