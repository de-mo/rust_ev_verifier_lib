use super::super::{
    common_types::{EncryptionGroup, ProofUnderline, SignatureJson},
    deserialize_seq_string_hex_to_seq_bigunit, implement_trait_verifier_data_json_decode,
    VerifierDataDecode,
};
use anyhow::anyhow;
use crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentPublicKeysPayload {
    pub encryption_group: EncryptionGroup,
    pub election_event_id: String,
    pub control_component_public_keys: ControlComponentPublicKeys,
    pub signature: SignatureJson,
}

implement_trait_verifier_data_json_decode!(ControlComponentPublicKeysPayload);

impl<'a> From<&'a ControlComponentPublicKeysPayload> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentPublicKeysPayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.control_component_public_keys),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for ControlComponentPublicKeysPayload {
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("OnlineCC keys"),
            HashableMessage::from(&self.control_component_public_keys.node_id),
            HashableMessage::from(&self.election_event_id),
        ]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::get_ca_cc(&self.control_component_public_keys.node_id).unwrap()
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentPublicKeys {
    pub node_id: usize,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub ccrj_choice_return_codes_encryption_public_key: Vec<BigUint>,
    pub ccrj_schnorr_proofs: Vec<ProofUnderline>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub ccmj_election_public_key: Vec<BigUint>,
    pub ccmj_schnorr_proofs: Vec<ProofUnderline>,
}

impl<'a> From<&'a ControlComponentPublicKeys> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentPublicKeys) -> Self {
        let mut elts = vec![
            Self::from(&value.node_id),
            Self::from(&value.ccrj_choice_return_codes_encryption_public_key),
        ];
        let ccrj: Vec<HashableMessage> = value.ccrj_schnorr_proofs.iter().map(Self::from).collect();
        elts.push(Self::from(ccrj));
        elts.push(Self::from(&value.ccmj_election_public_key));
        let ccmj: Vec<HashableMessage> = value.ccmj_schnorr_proofs.iter().map(Self::from).collect();
        elts.push(Self::from(ccmj));
        Self::from(elts)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::test::dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = dataset_tally_path()
            .join("setup")
            .join("controlComponentPublicKeysPayload.1.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = ControlComponentPublicKeysPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
