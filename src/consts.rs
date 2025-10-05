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

// Constants from specification
pub const MAXIMUM_NUMBER_OF_SUPPORTED_VOTING_OPTIONS_N_SUP: usize = 5000;
pub const MAXIMUM_SUPPORTED_NUMBER_OF_SELECTIONS_PSI_SUP: usize = 120;
pub const MAXIMUM_SUPPORTED_NUMBER_OF_WRITE_IN_OPTIONS: usize = 30;
pub const MAXIMUM_WRITE_IN_OPTION_LENGTH: usize = 500;
pub const MAXIMUM_ACTUAL_VOTING_OPTION_LENGTH: usize = 50;
pub const CHARACTER_LENGTH_OF_UNIQUE_IDENTIFIERS: usize = 32;

/// Env Variables
pub const ENV_VERIFIER_DATASET_PASSWORD: &str = "VERIFIER_DATASET_PASSWORD";
pub const ENV_TXT_REPORT_TAB_SIZE: &str = "TXT_REPORT_TAB_SIZE";
pub const ENV_REPORT_FORMAT_DATE: &str = "REPORT_FORMAT_DATE";
pub const ENV_DIRECT_TRUST_DIR_PATH: &str = "DIRECT_TRUST_DIR_PATH";
