use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};

use super::super::{
    common_types::{EncryptionParametersDef, Signature},
    implement_trait_verifier_data_json_decode, VerifierDataDecode,
};
use anyhow::anyhow;
use rust_ev_crypto_primitives::{
    ByteArray, EncryptionParameters, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TallyComponentVotesPayload {
    pub election_event_id: String,
    pub ballot_id: String,
    pub ballot_box_id: String,
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub votes: Vec<Vec<usize>>,
    pub actual_selected_voting_options: Vec<Vec<String>>,
    pub decoded_write_in_votes: Vec<Vec<String>>,
    pub signature: Signature,
}

implement_trait_verifier_data_json_decode!(TallyComponentVotesPayload);

impl VerifyDomainTrait for TallyComponentVotesPayload {}

impl<'a> From<&'a TallyComponentVotesPayload> for HashableMessage<'a> {
    fn from(value: &'a TallyComponentVotesPayload) -> Self {
        Self::from(vec![
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_id),
            Self::from(&value.ballot_box_id),
            Self::from(&value.encryption_group),
            Self::from(&value.votes),
            Self::from(&value.actual_selected_voting_options),
            Self::from(&value.decoded_write_in_votes),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for TallyComponentVotesPayload {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("decoded votes"),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.ballot_box_id),
        ]
    }

    fn get_certificate_authority(&self) -> anyhow::Result<String> {
        Ok(String::from(CertificateAuthority::SdmTally))
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[cfg(test)]
mod test {
    use super::{super::super::test::test_data_structure, *};
    use crate::config::test::{test_ballot_box_path, CONFIG_TEST};
    use std::fs;

    test_data_structure!(
        TallyComponentVotesPayload,
        "tallyComponentVotesPayload.json",
        test_ballot_box_path
    );
}
