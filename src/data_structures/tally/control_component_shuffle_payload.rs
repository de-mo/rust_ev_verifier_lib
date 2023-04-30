use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, SignatureJson},
    error::{DeserializeError, DeserializeErrorType},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use super::tally_component_shuffle_payload::VerifiableShuffle;
use crate::{
    data_structures::common_types::DecryptionProof,
    error::{create_verifier_error, VerifierError},
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentShufflePayload {
    pub encryption_group: EncryptionGroup,
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
    use std::fs;
    use std::path::Path;

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset1")
            .join("tally")
            .join("ballot_boxes")
            .join("9a19164550794441b25f7f744f2e91fb")
            .join("controlComponentShufflePayload_1.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ControlComponentShufflePayload::from_json(&json);
        println!("{:?}", r_eec.as_ref().err());
        assert!(r_eec.is_ok())
    }
}
