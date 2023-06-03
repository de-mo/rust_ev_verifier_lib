use super::super::{
    common_types::{EncryptionGroup, SignatureJson},
    deserialize_seq_seq_string_hex_to_seq_seq_bigunit, implement_trait_verifier_data_json_decode,
    VerifierDataDecode,
};
use crate::crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use anyhow::anyhow;
use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentTallyDataPayload {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub ballot_box_default_title: String,
    pub encryption_group: EncryptionGroup,
    pub verification_card_ids: Vec<String>,
    #[serde(deserialize_with = "deserialize_seq_seq_string_hex_to_seq_seq_bigunit")]
    pub verification_card_public_keys: Vec<Vec<BigUint>>,
    pub signature: SignatureJson,
}

implement_trait_verifier_data_json_decode!(SetupComponentTallyDataPayload);

impl<'a> From<&'a SetupComponentTallyDataPayload> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentTallyDataPayload) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.election_event_id));
        elts.push(Self::from(&value.verification_card_set_id));
        elts.push(Self::from(&value.ballot_box_default_title));
        elts.push(Self::from(&value.encryption_group));
        elts.push(Self::from(&value.verification_card_ids));
        let l: Vec<HashableMessage> = value
            .verification_card_public_keys
            .iter()
            .map(|p| HashableMessage::from(p))
            .collect();
        elts.push(Self::from(l));
        Self::from(elts)
    }
}

impl<'a> VerifiySignatureTrait<'a> for SetupComponentTallyDataPayload {
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("tally data"),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.verification_card_set_id),
        ]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::SdmConfig
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
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
            .join("setupComponentTallyDataPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = SetupComponentTallyDataPayload::from_json(&json);
        assert!(r_eec.is_ok())
    }
}
