use super::{
    super::{
        common_types::{EncryptionParametersDef, Signature},
        deserialize_seq_string_base64_to_seq_integer, implement_trait_verifier_data_json_decode,
        DataStructureError, VerifierDataDecode,
    },
    verifiable_shuffle::{verifiy_domain_for_verifiable_shuffle, VerifiableShuffle},
};
use crate::{
    data_structures::common_types::DecryptionProof,
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError},
};
use rust_ev_crypto_primitives::{
    elgamal::EncryptionParameters, ByteArray, HashableMessage, Integer, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TallyComponentShufflePayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub ballot_box_id: String,
    pub verifiable_shuffle: VerifiableShuffle,
    pub verifiable_plaintext_decryption: VerifiablePlaintextDecryption,
    pub signature: Signature,
}
implement_trait_verifier_data_json_decode!(TallyComponentShufflePayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiablePlaintextDecryption {
    pub decrypted_votes: Vec<DecryptedVote>,
    pub decryption_proofs: Vec<DecryptionProof>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedVote {
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub message: Vec<Integer>,
}

impl VerifyDomainTrait<String> for TallyComponentShufflePayload {
    fn new_domain_verifications() -> rust_ev_crypto_primitives::DomainVerifications<Self, String> {
        let mut res = rust_ev_crypto_primitives::DomainVerifications::default();
        res.add_verification(|v: &Self| {
            verifiy_domain_for_verifiable_shuffle(&v.verifiable_shuffle)
        });
        res
    }
}

impl<'a> From<&'a TallyComponentShufflePayload> for HashableMessage<'a> {
    fn from(value: &'a TallyComponentShufflePayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_box_id),
            Self::from(&value.verifiable_shuffle),
            Self::from(
                value
                    .verifiable_plaintext_decryption
                    .decrypted_votes
                    .iter()
                    .map(|v| HashableMessage::from(&v.message))
                    .collect::<Vec<Self>>(),
            ),
            Self::from(
                value
                    .verifiable_plaintext_decryption
                    .decryption_proofs
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for TallyComponentShufflePayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Box<VerifySignatureError>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("shuffle"),
            HashableMessage::from("offline"),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.ballot_box_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmTally)
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[cfg(test)]
mod test_one {
    use super::{
        super::super::test::{
            test_data_structure, test_data_structure_read_data_set,
            test_data_structure_verify_domain, test_data_structure_verify_signature,
        },
        *,
    };
    use crate::config::test::{
        test_ballot_box_many_votes_path, test_ballot_box_one_vote_path,
        test_ballot_box_zero_vote_path, CONFIG_TEST,
    };
    use paste::paste;
    use std::fs;

    test_data_structure!(
        one_vote,
        TallyComponentShufflePayload,
        "tallyComponentShufflePayload.json",
        test_ballot_box_one_vote_path
    );

    test_data_structure!(
        zero_vote,
        TallyComponentShufflePayload,
        "tallyComponentShufflePayload.json",
        test_ballot_box_zero_vote_path
    );

    test_data_structure!(
        many_votes,
        TallyComponentShufflePayload,
        "tallyComponentShufflePayload.json",
        test_ballot_box_many_votes_path
    );

    #[test]
    fn test_argument_correct() {
        let data = get_data_res_many_votes().unwrap();
        assert!(data
            .verifiable_shuffle
            .shuffle_argument
            .product_argument
            .c_b
            .is_some());
        assert!(data
            .verifiable_shuffle
            .shuffle_argument
            .product_argument
            .hadamard_argument
            .is_some());
        let data = get_data_res_one_vote().unwrap();
        assert!(data
            .verifiable_shuffle
            .shuffle_argument
            .product_argument
            .c_b
            .is_none());
        assert!(data
            .verifiable_shuffle
            .shuffle_argument
            .product_argument
            .hadamard_argument
            .is_none());
        let data = get_data_res_zero_vote().unwrap();
        assert!(data
            .verifiable_shuffle
            .shuffle_argument
            .product_argument
            .c_b
            .is_none());
        assert!(data
            .verifiable_shuffle
            .shuffle_argument
            .product_argument
            .hadamard_argument
            .is_none());
    }
}
