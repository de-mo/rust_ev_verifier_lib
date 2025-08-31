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
        implement_trait_verifier_data_json_decode, DataStructureError, DataStructureErrorImpl,
        VerifierDataDecode,
    },
    VerifierTallyDataType,
};
use crate::{
    data_structures::{VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{CertificateAuthority, VerifiyJSONSignatureTrait, VerifiySignatureTrait},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, ByteArray, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;
use std::sync::Arc;

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

impl VerifierDataToTypeTrait for TallyComponentVotesPayload {
    fn data_type() -> VerifierDataType {
        VerifierDataType::Tally(VerifierTallyDataType::TallyComponentVotesPayload)
    }
}

implement_trait_verifier_data_json_decode!(TallyComponentVotesPayload);

impl VerifyDomainTrait<String> for TallyComponentVotesPayload {}

impl<'a> From<&'a TallyComponentVotesPayload> for HashableMessage<'a> {
    fn from(value: &'a TallyComponentVotesPayload) -> Self {
        let mut res = vec![
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_id),
            Self::from(&value.ballot_box_id),
            Self::from(&value.encryption_group),
        ];
        if !value.votes.is_empty() {
            res.push(Self::from(value.votes.as_slice()))
        };
        if !value.actual_selected_voting_options.is_empty() {
            res.push(Self::from(value.actual_selected_voting_options.as_slice()))
        }
        if !value.decoded_write_in_votes.is_empty() {
            res.push(Self::from(value.decoded_write_in_votes.as_slice()))
        }
        Self::from(res)
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

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
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
            test_data_structure, test_data_structure_read_data_set,
            test_data_structure_verify_domain, test_data_structure_verify_signature,
        },
        *,
    };
    use crate::config::test::{
        test_ballot_box_one_vote_path, test_ballot_box_zero_vote_path, CONFIG_TEST,
    };
    use std::fs;

    test_data_structure!(
        TallyComponentVotesPayload,
        "tallyComponentVotesPayload.json",
        test_ballot_box_one_vote_path
    );

    #[test]
    fn test_signature_empty_votes() {
        let json = fs::read_to_string(
            test_ballot_box_zero_vote_path().join("tallyComponentVotesPayload.json"),
        )
        .unwrap();
        let data = TallyComponentVotesPayload::decode_json(&json).unwrap();
        let ks = CONFIG_TEST.keystore().unwrap();
        let sign_validate_res = data.verify_signatures(&ks);
        for r in sign_validate_res {
            assert!(r.is_ok());
            assert!(r.unwrap())
        }
    }
}
