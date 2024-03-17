use super::super::{
    common_types::{EncryptionParametersDef, Signature},
    deserialize_seq_seq_string_base64_to_seq_seq_integer,
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};
use anyhow::anyhow;
use rug::Integer;
use rust_ev_crypto_primitives::{ByteArray, EncryptionParameters, HashableMessage};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentTallyDataPayload {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub ballot_box_default_title: String,
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub verification_card_ids: Vec<String>,
    #[serde(deserialize_with = "deserialize_seq_seq_string_base64_to_seq_seq_integer")]
    pub verification_card_public_keys: Vec<Vec<Integer>>,
    pub signature: Signature,
}

implement_trait_verifier_data_json_decode!(SetupComponentTallyDataPayload);

impl<'a> From<&'a SetupComponentTallyDataPayload> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentTallyDataPayload) -> Self {
        let mut elts = vec![
            Self::from(&value.election_event_id),
            Self::from(&value.verification_card_set_id),
            Self::from(&value.ballot_box_default_title),
            Self::from(&value.encryption_group),
            Self::from(&value.verification_card_ids),
        ];
        let l: Vec<HashableMessage> = value
            .verification_card_public_keys
            .iter()
            .map(HashableMessage::from)
            .collect();
        elts.push(Self::from(l));
        Self::from(elts)
    }
}

impl<'a> VerifiySignatureTrait<'a> for SetupComponentTallyDataPayload {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("tally data"),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.verification_card_set_id),
        ]
    }

    fn get_certificate_authority(&self) -> anyhow::Result<String> {
        Ok(String::from(CertificateAuthority::SdmConfig))
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_verification_card_set_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = test_verification_card_set_path().join("setupComponentTallyDataPayload.json");
        let json = fs::read_to_string(path).unwrap();
        let r_eec = SetupComponentTallyDataPayload::from_json(&json);
        if r_eec.is_err() {
            println!("{:?}", r_eec.as_ref().unwrap_err());
        }
        assert!(r_eec.is_ok())
    }
}
