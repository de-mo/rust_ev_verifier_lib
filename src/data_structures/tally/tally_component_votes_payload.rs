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
        DataStructureError, DataStructureErrorImpl, VerifierDataDecode,
        common_types::{EncryptionParametersDef, Signature},
        implement_trait_verifier_data_json_decode,
    },
    VerifierTallyDataType,
};
use crate::{
    data_structures::{VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{CertificateAuthority, VerifiyJSONSignatureTrait, VerifiySignatureTrait},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    ByteArray, HashableMessage, VerifyDomainTrait, elgamal::EncryptionParameters,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TallyComponentVotesPayload {
    pub election_event_id: String,
    pub ballot_box_id: String,
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    #[serde(alias = "votes")]
    pub decrypted_votes: Vec<Vec<usize>>,
    #[serde(alias = "actualSelectedVotingOptions")]
    pub decoded_votes: Vec<Vec<String>>,
    #[serde(alias = "decodedWriteInVotes")]
    pub decoded_write_ins: Vec<Vec<String>>,
    pub signature: Option<Signature>,
}

impl VerifierDataToTypeTrait for TallyComponentVotesPayload {
    fn data_type() -> VerifierDataType {
        VerifierDataType::Tally(VerifierTallyDataType::TallyComponentVotesPayload)
    }
}

implement_trait_verifier_data_json_decode!(TallyComponentVotesPayload);

impl VerifyDomainTrait<String> for TallyComponentVotesPayload {}

impl<'a> From<&'a TallyComponentVotesPayload> for HashableMessage<'a> {
    fn from(value: &'a TallyComponentVotesPayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_box_id),
            Self::from(value.decrypted_votes.as_slice()),
            Self::from(value.decoded_votes.as_slice()),
            Self::from(value.decoded_write_ins.as_slice()),
        ])
    }
}

impl<'a> VerifiyJSONSignatureTrait<'a> for TallyComponentVotesPayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("decoded votes"),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.ballot_box_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmTally)
    }

    fn get_signature(&self) -> Option<ByteArray> {
        self.signature.as_ref().map(|s| s.get_signature())
    }
}

impl<'a> VerifiySignatureTrait<'a> for TallyComponentVotesPayload {
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
        test_data_signature_hash_path,
    };
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
        EncodeTrait, RecursiveHashTrait,
    };
    use std::fs;

    test_data_structure!(
        TallyComponentVotesPayload,
        "tallyComponentVotesPayload.json",
        test_ballot_box_one_vote_path
    );

    test_hash_json!(
        TallyComponentVotesPayload,
        "verify-signature-tally-component-votes.json"
    );

    #[test]
    fn test_signature_empty_votes() {
        let json = fs::read_to_string(
            test_ballot_box_zero_vote_path().join("tallyComponentVotesPayload.json"),
        )
        .unwrap();
        let data = TallyComponentVotesPayload::decode_json(&json).unwrap();
        let ks = get_keystore();
        let sign_validate_res = data.verify_signatures(&ks);
        for r in sign_validate_res {
            assert!(r.is_ok());
            assert!(r.unwrap())
        }
    }
}
