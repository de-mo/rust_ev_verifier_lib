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

mod v0501_encryption_parameters;
mod v0502_verify_small_prime_group_members;
mod v0503_voting_options;
mod v0504_key_generation_schnorr_proofs;

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
            "05.01",
            "VerifyEncryptionParameters",
            v0501_encryption_parameters::fn_0501_verify_encryption_parameters,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyEncryptionParameters",
            source: Box::new(e),
        })?,
        Verification::new(
            "05.02",
            "VerifySmallPrimeGroupMembers",
            v0502_verify_small_prime_group_members::fn_0502_verify_small_prime_group_members,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySmallPrimeGroupMembers",
            source: Box::new(e),
        })?,
        Verification::new(
            "05.03",
            "VerifyVotingOptions",
            v0503_voting_options::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifyVotingOptions",
            source: Box::new(e),
        })?,
        Verification::new(
            "05.04",
            "VerifySchnorrProofs",
            v0504_key_generation_schnorr_proofs::fn_verification,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySchnorrProofs",
            source: Box::new(e),
        })?,
    ]))
}
