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

use super::verifiable_shuffle::VerifiableShuffle;
use super::VerifierTallyDataType;
use super::{
    super::{
        common_types::{EncryptionParametersDef, Signature},
        deserialize_seq_ciphertext, implement_trait_verifier_data_json_decode, DataStructureError,
        DataStructureErrorImpl, VerifierDataDecode,
    },
    verifiable_shuffle::verifiy_domain_for_verifiable_shuffle,
};
use crate::data_structures::{VerifierDataToTypeTrait, VerifierDataType};
use crate::direct_trust::VerifiySignatureTrait;
use crate::{
    data_structures::common_types::DecryptionProof,
    direct_trust::{CertificateAuthority, VerifiyJSONSignatureTrait},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::DomainVerifications;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::{Ciphertext, EncryptionParameters},
    ByteArray, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentShufflePayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub ballot_box_id: String,
    pub node_id: usize,
    pub verifiable_shuffle: VerifiableShuffle,
    pub verifiable_decryptions: VerifiableDecryptions,
    pub signature: Option<Signature>,
}

impl VerifierDataToTypeTrait for ControlComponentShufflePayload {
    fn data_type() -> VerifierDataType {
        VerifierDataType::Tally(VerifierTallyDataType::ControlComponentShufflePayload)
    }
}

implement_trait_verifier_data_json_decode!(ControlComponentShufflePayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiableDecryptions {
    #[serde(deserialize_with = "deserialize_seq_ciphertext")]
    pub ciphertexts: Vec<Ciphertext>,
    pub decryption_proofs: Vec<DecryptionProof>,
}

impl VerifyDomainTrait<String> for ControlComponentShufflePayload {
    fn new_domain_verifications() -> DomainVerifications<Self, String> {
        let mut res = DomainVerifications::default();
        res.add_verification(|v: &Self| {
            verifiy_domain_for_verifiable_shuffle(&v.verifiable_shuffle)
        });
        res
    }
}

impl<'a> From<&'a ControlComponentShufflePayload> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentShufflePayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_box_id),
            Self::from(&value.node_id),
            Self::from(&value.verifiable_shuffle),
            Self::from(&value.verifiable_decryptions),
        ])
    }
}

impl<'a> From<&'a VerifiableDecryptions> for HashableMessage<'a> {
    fn from(value: &'a VerifiableDecryptions) -> Self {
        Self::from(vec![
            Self::from(
                value
                    .ciphertexts
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
            Self::from(
                value
                    .decryption_proofs
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
        ])
    }
}

impl<'a> VerifiyJSONSignatureTrait<'a> for ControlComponentShufflePayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("shuffle"),
            HashableMessage::from(&self.node_id),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.ballot_box_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        CertificateAuthority::get_ca_cc(&self.node_id)
    }

    fn get_signature(&self) -> Option<ByteArray> {
        self.signature.as_ref().map(|s| s.get_signature())
    }
}

impl<'a> VerifiySignatureTrait<'a> for ControlComponentShufflePayload {
    fn verifiy_signature(
        &'a self,
        keystore: &crate::direct_trust::Keystore,
    ) -> Result<bool, crate::direct_trust::VerifySignatureError> {
        self.verifiy_json_signature(keystore)
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::test::{
            file_to_test_cases, json_to_hashable_message, json_to_testdata, test_data_structure,
            test_data_structure_read_data_set, test_data_structure_verify_domain,
            test_data_structure_verify_signature, test_hash_json,
        },
        *,
    };
    use crate::config::test::{
        get_keystore, test_ballot_box_many_votes_path, test_ballot_box_one_vote_path,
        test_ballot_box_zero_vote_path, test_data_signature_hash_path,
    };
    use paste::paste;
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
        EncodeTrait, RecursiveHashTrait,
    };
    use std::fs;

    test_data_structure!(
        one_vote,
        ControlComponentShufflePayload,
        "controlComponentShufflePayload_1.json",
        test_ballot_box_one_vote_path
    );

    test_data_structure!(
        many_votes,
        ControlComponentShufflePayload,
        "controlComponentShufflePayload_1.json",
        test_ballot_box_many_votes_path
    );

    test_data_structure!(
        no_vote,
        ControlComponentShufflePayload,
        "controlComponentShufflePayload_1.json",
        test_ballot_box_zero_vote_path
    );

    test_hash_json!(
        ControlComponentShufflePayload,
        "verify-signature-control-component-shuffle.json"
    );
}
