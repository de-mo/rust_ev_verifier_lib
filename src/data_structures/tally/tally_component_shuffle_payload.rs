// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use super::{
    super::{
        common_types::{EncryptionParametersDef, Signature},
        deserialize_seq_string_base64_to_seq_integer, implement_trait_verifier_data_json_decode,
        DataStructureError, DataStructureErrorImpl, VerifierDataDecode,
    },
    verifiable_shuffle::{verifiy_domain_for_verifiable_shuffle, VerifiableShuffle},
    VerifierTallyDataType,
};
use crate::{
    data_structures::{common_types::DecryptionProof, VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{
        CertificateAuthority, VerifiyJSONSignatureTrait, VerifiySignatureTrait,
    },
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, ByteArray, DomainVerifications, HashableMessage, Integer,
    VerifyDomainTrait,
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

impl VerifierDataToTypeTrait for TallyComponentShufflePayload {
    fn data_type() -> VerifierDataType {
        VerifierDataType::Tally(VerifierTallyDataType::TallyComponentShufflePayload)
    }
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
    fn new_domain_verifications() -> DomainVerifications<Self, String> {
        let mut res = DomainVerifications::default();
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
                    .map(|v| HashableMessage::from(v.message.as_slice()))
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

impl<'a> VerifiyJSONSignatureTrait<'a> for TallyComponentShufflePayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError> {
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

impl<'a> VerifiySignatureTrait<'a> for TallyComponentShufflePayload {
    fn verifiy_signature(
        &'a self,
        keystore: &crate::direct_trust::Keystore,
    ) -> Result<bool, crate::direct_trust::VerifySignatureError> {
        self.verifiy_json_signature(keystore)
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
