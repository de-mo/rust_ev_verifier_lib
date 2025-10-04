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

use crate::{
    data_structures::{
        context::{
            election_event_configuration::ElectionEventConfigurationData,
            election_event_context_payload::ElectionEventContext,
        },
        tally::ech_0222::{ECH0222Data, ECh0222differencesTrait},
    },
    file_structure::tally_directory::BBDirectoryTrait,
    verification::VerificationResult,
};

pub fn verify_ech0222<B: BBDirectoryTrait>(
    election_event_context: &ElectionEventContext,
    election_event_configuration: &ElectionEventConfigurationData,
    ech_0222: &ECH0222Data,
    bb_directories: &[B],
) -> VerificationResult {
    let mut result = VerificationResult::new();
    let ech_0222_prime =
        match crate::data_structures::tally::ech_0222::ECH0222Data::create_ech0222_data(
            election_event_context,
            election_event_configuration,
            bb_directories,
        ) {
            Ok(d) => d,
            Err(e) => {
                result.push(
                    crate::verification::VerificationEvent::new_error_from_error(&e)
                        .add_context("ECH0222 data cannot be calculated"),
                );
                return result;
            }
        };
    let diff = ech_0222.calculate_differences(&ech_0222_prime);
    diff.iter().for_each(|d| {
        result.push(crate::verification::VerificationEvent::new_failure(
            &format!("ECH0222 difference found: {}", d),
        ))
    });
    result
}
