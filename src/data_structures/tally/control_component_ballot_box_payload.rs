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
        common_types::{CiphertextDef, EncryptionParametersDef, Signature},
        implement_trait_verifier_data_json_decode, DataStructureError, DataStructureErrorImpl,
        VerifierDataDecode,
    },
    VerifierTallyDataType,
};
use crate::{
    data_structures::{
        common_types::{DecryptionProof, SchnorrProof},
        VerifierDataToTypeTrait, VerifierDataType,
    },
    direct_trust::{CertificateAuthority, VerifiyJSONSignatureTrait, VerifiySignatureTrait},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::{Ciphertext, EncryptionParameters},
    ByteArray, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentBallotBoxPayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub ballot_box_id: String,
    pub node_id: usize,
    pub confirmed_encrypted_votes: Vec<ConfirmedEncryptedVote>,
    pub signature: Option<Signature>,
}

impl VerifierDataToTypeTrait for ControlComponentBallotBoxPayload {
    fn data_type() -> VerifierDataType {
        VerifierDataType::Tally(VerifierTallyDataType::ControlComponentBallotBoxPayload)
    }
}

implement_trait_verifier_data_json_decode!(ControlComponentBallotBoxPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedEncryptedVote {
    pub context_ids: ContextIds,
    #[serde(with = "CiphertextDef")]
    pub encrypted_vote: Ciphertext,
    #[serde(with = "CiphertextDef")]
    pub exponentiated_encrypted_vote: Ciphertext,
    #[serde(with = "CiphertextDef")]
    pub encrypted_partial_choice_return_codes: Ciphertext,
    pub exponentiation_proof: SchnorrProof,
    pub plaintext_equality_proof: DecryptionProof,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextIds {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub verification_card_id: String,
}

impl VerifyDomainTrait<String> for ControlComponentBallotBoxPayload {}

impl<'a> From<&'a ControlComponentBallotBoxPayload> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentBallotBoxPayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_box_id),
            Self::from(&value.node_id),
            Self::from(
                value
                    .confirmed_encrypted_votes
                    .iter()
                    .map(Self::from)
                    .collect::<Vec<_>>(),
            ),
        ])
    }
}

impl<'a> From<&'a ConfirmedEncryptedVote> for HashableMessage<'a> {
    fn from(value: &'a ConfirmedEncryptedVote) -> Self {
        Self::from(vec![
            Self::from(&value.context_ids),
            Self::from(&value.encrypted_vote),
            Self::from(&value.exponentiated_encrypted_vote),
            Self::from(&value.encrypted_partial_choice_return_codes),
            Self::from(&value.exponentiation_proof),
            Self::from(&value.plaintext_equality_proof),
        ])
    }
}

impl<'a> From<&'a ContextIds> for HashableMessage<'a> {
    fn from(value: &'a ContextIds) -> Self {
        Self::from(vec![
            Self::from(&value.election_event_id),
            Self::from(&value.verification_card_set_id),
            Self::from(&value.verification_card_id),
        ])
    }
}

impl<'a> VerifiyJSONSignatureTrait<'a> for ControlComponentBallotBoxPayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("ballotbox"),
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

impl<'a> VerifiySignatureTrait<'a> for ControlComponentBallotBoxPayload {
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
        get_keystore, test_ballot_box_one_vote_path, test_ballot_box_zero_vote_path,
        test_resources_path,
    };
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
        EncodeTrait, RecursiveHashTrait,
    };
    use std::fs;

    test_data_structure!(
        ControlComponentBallotBoxPayload,
        "controlComponentBallotBoxPayload_1.json",
        test_ballot_box_one_vote_path
    );

    test_hash_json!(
        ControlComponentBallotBoxPayload,
        "verify-signature-control-component-ballot-box.json"
    );

    #[test]
    fn test_signature_empty_votes() {
        let json = fs::read_to_string(
            test_ballot_box_zero_vote_path().join("controlComponentBallotBoxPayload_4.json"),
        )
        .unwrap();
        let data = ControlComponentBallotBoxPayload::decode_json(&json).unwrap();
        let ks = get_keystore();
        let sign_validate_res = data.verify_signatures(&ks);
        for r in sign_validate_res {
            assert!(r.is_ok());
            assert!(r.unwrap())
        }
    }
}
