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

mod v0801_verify_confirmed_encrypted_votes_consistency;
mod v0802_verify_ciphertexts_consistency;
mod v0803_verify_plaintexts_consistency;
mod v0804_verify_verification_card_ids_consistency;
mod v0805_verify_ballot_box_ids_consistency;
mod v0806_verify_file_name_ballot_box_ids_consistency;
mod v0807_verify_number_confirmed_encrypted_votes_consistency;
mod v0808_verify_election_event_id_consistency;
mod v0809_verify_node_ids_consistency;
mod v0810_verify_file_name_node_ids_consistency;
mod v0811_verify_encryption_group_consistency;

use super::super::{suite::VerificationList, verifications::Verification};
use crate::{
    config::VerifierConfig,
    verification::{meta_data::VerificationMetaDataList, VerificationError, VerificationErrorImpl},
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static VerifierConfig,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![
        Verification::new(
            "08.01",
            "VerifyConfirmedEncryptedVotesConsistency",
            v0801_verify_confirmed_encrypted_votes_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyConfirmedEncryptedVotesConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.02",
            "VerifyCiphertextsConsistency",
            v0802_verify_ciphertexts_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyCiphertextsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.03",
            "VerifyPlaintextsConsistency",
            v0803_verify_plaintexts_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyPlaintextsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.04",
            "VerifyVerificationCardIdsConsistency",
            v0804_verify_verification_card_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyVerificationCardIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.05",
            "VerifyBallotBoxIdsConsistency",
            v0805_verify_ballot_box_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyBallotBoxIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.06",
            "VerifyFileNameBallotBoxIdsConsistency",
            v0806_verify_file_name_ballot_box_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyFileNameBallotBoxIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.07",
            "VerifyNumberConfirmedEncryptedVotesConsistency",
            v0807_verify_number_confirmed_encrypted_votes_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyNumberConfirmedEncryptedVotesConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.08",
            "VerifyElectionEventIdConsistency",
            v0808_verify_election_event_id_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyElectionEventIdConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.09",
            "VerifyNodeIdsConsistency",
            v0809_verify_node_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyNodeIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.10",
            "VerifyFileNameNodeIdsConsistency",
            v0810_verify_file_name_node_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyFileNameNodeIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "08.11",
            "VerifyEncryptionGroupConsistency",
            v0811_verify_encryption_group_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyEncryptionGroupConsistency",
            source: Box::new(e),
        })?,
    ]))
}
