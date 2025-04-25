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

mod v1001_verify_online_control_components;
mod v1002_verify_tally_control_component;

use super::super::{suite::VerificationList, verifications::Verification};
use crate::{
    config::VerifierConfig,
    verification::{meta_data::VerificationMetaDataList, VerificationError},
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static VerifierConfig,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![
        Verification::new(
            "10.01",
            "VerifyOnlineControlComponents",
            v1001_verify_online_control_components::fn_verification,
            metadata_list,
            config,
        )?,
        Verification::new(
            "10.02",
            "VerifyTallyControlComponent",
            v1002_verify_tally_control_component::fn_verification,
            metadata_list,
            config,
        )?,
    ]))
}
