use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, SignatureJson},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use super::tally_component_shuffle_payload::VerifiableShuffle;
use crate::data_structures::common_types::DecryptionProof;
use anyhow::anyhow;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ControlComponentShufflePayload {
    pub(crate) encryption_group: EncryptionGroup,
    pub(crate) election_event_id: String,
    pub(crate) ballot_box_id: String,
    pub(crate) node_id: usize,
    pub(crate) verifiable_decryptions: VerifiableDecryptions,
    pub(crate) verifiable_shuffle: VerifiableShuffle,
    pub(crate) signature: SignatureJson,
}
implement_trait_verifier_data_json_decode!(ControlComponentShufflePayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VerifiableDecryptions {
    pub(crate) ciphertexts: Vec<ExponentiatedEncryptedElement>,
    pub(crate) decryption_proofs: Vec<DecryptionProof>,
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
            .join("controlComponentShufflePayload_1.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = ControlComponentShufflePayload::from_json(&json);
        println!("{:?}", r_eec.as_ref().err());
        assert!(r_eec.is_ok())
    }
}
