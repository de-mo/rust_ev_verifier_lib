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

mod v0301_encryption_group_consistency;
mod v0302_verify_node_ids_consistency;
mod v0303_file_name_node_ids_consistency;
mod v0304_election_event_id_consistency;
mod v0305_verification_card_set_ids_consistency;
mod v0306_file_name_verification_card_set_ids_consistency;
mod v0307_verification_card_ids_consistency;
mod v0308_ccr_choice_return_codes_pk_consistency;
mod v0309_ccm_election_pk_consistency;
mod v0310_ccm_and_ccr_schnorr_proofs_consistency;
mod v0311_choice_return_codes_public_key_consistency;
mod v0312_election_pk_consistency;
mod v0313_primes_mapping_table_consistency;
mod v0314_total_voters_consistency;

use super::super::{
    meta_data::VerificationMetaDataList, suite::VerificationList, verifications::Verification,
};
use crate::{
    config::VerifierConfig,
    verification::{VerificationError, VerificationErrorImpl},
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static VerifierConfig,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![
        Verification::new(
            "03.01",
            "VerifyEncryptionGroupConsistency",
            v0301_encryption_group_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyEncryptionGroupConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.02",
            "VerifyNodeIdsConsistency",
            v0302_verify_node_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyNodeIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.03",
            "VerifyFileNameNodeIdsConsistency",
            v0303_file_name_node_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyFileNameNodeIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.04",
            "VerifyElectionEventIdConsistency",
            v0304_election_event_id_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyElectionEventIdConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.05",
            "VerifyVerificationCardSetIdsConsistency",
            v0305_verification_card_set_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyVerificationCardSetIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.06",
            "VerifyFileNameVerificationCardSetIdsConsistency",
            v0306_file_name_verification_card_set_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyFileNameVerificationCardSetIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.07",
            "VerifyVerificationCardIdsConsistency",
            v0307_verification_card_ids_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyVerificationCardIdsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.08",
            "VerifyCCRChoiceReturnCodesPublicKeyConsistency",
            v0308_ccr_choice_return_codes_pk_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyCCRChoiceReturnCodesPublicKeyConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.09",
            "VerifyCCMElectionPublicKeyConsistency",
            v0309_ccm_election_pk_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyCCMElectionPublicKeyConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.10",
            "VerifyCCMAndCCRSchnorrProofsConsistency",
            v0310_ccm_and_ccr_schnorr_proofs_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyCCMAndCCRSchnorrProofsConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.11",
            "VerifyChoiceReturnCodesPublicKeyConsistency",
            v0311_choice_return_codes_public_key_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyChoiceReturnCodesPublicKeyConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.12",
            "VerifyElectionPublicKeyConsistency",
            v0312_election_pk_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyElectionPublicKeyConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.13",
            "VerifyPrimesMappingTableConsistency",
            v0313_primes_mapping_table_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyPrimesMappingTableConsistency",
            source: Box::new(e),
        })?,
        Verification::new(
            "03.14",
            "VerifyTotalVotersConsistency",
            v0314_total_voters_consistency::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyTotalVotersConsistency",
            source: Box::new(e),
        })?,
    ]))
}
