use super::super::{
    common_types::{ExponentiatedEncryptedElement, SignatureJson, EncryptionParametersDef},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use super::tally_component_shuffle_payload::VerifiableShuffle;
use crate::data_structures::common_types::DecryptionProof;
use anyhow::anyhow;
use rust_ev_crypto_primitives::EncryptionParameters;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentShufflePayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub ballot_box_id: String,
    pub node_id: usize,
    pub verifiable_decryptions: VerifiableDecryptions,
    pub verifiable_shuffle: VerifiableShuffle,
    pub signature: SignatureJson,
}
implement_trait_verifier_data_json_decode!(ControlComponentShufflePayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiableDecryptions {
    pub ciphertexts: Vec<ExponentiatedEncryptedElement>,
    pub decryption_proofs: Vec<DecryptionProof>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = test_dataset_tally_path()
            .join("tally")
            .join("ballot_boxes")
            .join("4AB4F95B8114C1DFEDB9586ADBFE36B3")
            .join("controlComponentShufflePayload_1.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = ControlComponentShufflePayload::from_json(&json);
        println!("{:?}", r_eec.as_ref().err());
        assert!(r_eec.is_ok())
    }
}
