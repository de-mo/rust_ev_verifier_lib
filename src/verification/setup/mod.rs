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

//! Module implementing the verifications for setup
mod authenticity;
mod completness;
mod consistency;
mod evidence;
mod integrity;

use super::{
    meta_data::VerificationMetaDataList, suite::VerificationList, VerificationError,
    VerificationErrorImpl,
};
use crate::config::VerifierConfig;

/// Collect the verifications of the submodules
pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static VerifierConfig,
) -> Result<VerificationList<'a>, VerificationError> {
    let mut res = VerificationList(vec![]);
    res.0.append(
        &mut authenticity::get_verifications(metadata_list, config)
            .map_err(|e| VerificationErrorImpl::GetCategory {
                category: "Authenticity",
                source: Box::new(e),
            })?
            .0,
    );
    res.0.append(
        &mut completness::get_verifications(metadata_list, config)
            .map_err(|e| VerificationErrorImpl::GetCategory {
                category: "Completness",
                source: Box::new(e),
            })?
            .0,
    );
    res.0.append(
        &mut consistency::get_verifications(metadata_list, config)
            .map_err(|e| VerificationErrorImpl::GetCategory {
                category: "Consistency",
                source: Box::new(e),
            })?
            .0,
    );
    res.0.append(
        &mut evidence::get_verifications(metadata_list, config)
            .map_err(|e| VerificationErrorImpl::GetCategory {
                category: "Evidence",
                source: Box::new(e),
            })?
            .0,
    );
    res.0.append(
        &mut integrity::get_verifications(metadata_list, config)
            .map_err(|e| VerificationErrorImpl::GetCategory {
                category: "Integrity",
                source: Box::new(e),
            })?
            .0,
    );
    Ok(res)
}
