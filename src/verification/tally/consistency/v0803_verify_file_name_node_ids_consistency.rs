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

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{
        VerificationDirectoryTrait,
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
    },
};

fn verify_nod_ir_for_tally_bb_dir<B: BBDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    for (i, f) in dir.control_component_ballot_box_payload_iter() {
        match f {
            Ok(s) => {
                if s.node_id != i {
                    result.push_with_context(
                        VerificationEvent::new_failure(&format!("node id {} for control_component_ballot_box_payload.{} not same than index", s.node_id, i)),
                        format!("{}/control_component_ballot_box_payload.{}", dir.name(), i),
                    )
                }
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "{}/control_component_ballot_box_payload.{} has wrong format",
                    dir.name(),
                    i
                ),
            )),
        }
    }

    for (i, f) in dir.control_component_shuffle_payload_iter() {
        match f {
            Ok(s) => {
                if s.node_id != i {
                    result.push_with_context(
                        VerificationEvent::new_failure(&format!("node id {} for control_component_shuffle_payload.{} not same than index", s.node_id, i)),
                        format!("{}/control_component_shuffle_payload.{}", dir.name(), i),
                    )
                }
            }
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "{}/control_component_shuffle_payload.{} has wrong format",
                    dir.name(),
                    i
                ),
            )),
        }
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();

    for bb in tally_dir.bb_directories().iter() {
        verify_nod_ir_for_tally_bb_dir(bb, result);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        config::test::{
            CONFIG_TEST, get_test_verifier_tally_dir as get_verifier_dir, test_data_path,
        },
        file_structure::VerificationDirectory,
        verification::VerificationPeriod,
    };

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn change_node_id() {
        let test_dir_path = test_data_path().join("verification_0803");
        for p in test_dir_path
            .read_dir()
            .unwrap()
            .map(|f| f.unwrap().path())
            .filter(|f| f.is_dir())
        {
            let dir = VerificationDirectory::new(&VerificationPeriod::Tally, &p);
            let mut result = VerificationResult::new();
            fn_verification(&dir, &CONFIG_TEST, &mut result);
            assert!(
                !result.has_errors(),
                "Has errors: path={} / errors={:?}",
                p.file_name().unwrap().to_str().unwrap(),
                result.errors()
            );
            assert!(
                result.has_failures(),
                "Has no failures: path={}",
                p.file_name().unwrap().to_str().unwrap()
            );
        }
    }
}
