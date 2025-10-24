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
        context_directory::ContextDirectoryTrait,
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
    },
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::elgamal::EncryptionParameters;

fn verify_encryption_group(
    eg: &EncryptionParameters,
    expected: &EncryptionParameters,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    if eg.p() != expected.p() {
        result.push(VerificationEvent::new_failure("p not equal"));
    }
    if eg.q() != expected.q() {
        result.push(VerificationEvent::new_failure("q not equal"));
    }
    if eg.g() != expected.g() {
        result.push(VerificationEvent::new_failure("g not equal"));
    }
    result
}

fn verify_encryption_group_for_tally_bb_dir<B: BBDirectoryTrait>(
    dir: &B,
    eg: &EncryptionParameters,
    result: &mut VerificationResult,
) {
    for (i, f) in dir.control_component_ballot_box_payload_iter() {
        match f {
            Ok(s) => result.append_with_context(
                &verify_encryption_group(&s.encryption_group, eg),
                format!("{}/control_component_ballot_box_payload.{}", dir.name(), i),
            ),
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
            Ok(s) => result.append_with_context(
                &verify_encryption_group(&s.encryption_group, eg),
                format!("{}/control_component_shuffle_payload.{}", dir.name(), i),
            ),
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!(
                    "{}/control_component_shuffle_payload.{} has wrong format",
                    dir.name(),
                    i
                ),
            )),
        }
    }

    match dir.tally_component_shuffle_payload() {
        Ok(s) => result.append_with_context(
            &verify_encryption_group(&s.encryption_group, eg),
            format!("{}/tally_component_shuffle_payload", dir.name()),
        ),
        Err(e) => result.push(
            VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/tally_component_shuffle_payload has wrong format",
                dir.name()
            )),
        ),
    }

    match dir.tally_component_votes_payload() {
        Ok(s) => result.append_with_context(
            &verify_encryption_group(&s.encryption_group, eg),
            format!("{}/tally_component_votes_payload", dir.name()),
        ),
        Err(e) => result.push(
            VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/tally_component_votes_payload has wrong format",
                dir.name()
            )),
        ),
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let config_dir = dir.context();
    let tally_dir = dir.unwrap_tally();
    let ee_context_payload = match config_dir.election_event_context_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    let eg = &ee_context_payload.encryption_group;

    for bb in tally_dir.bb_directories().iter() {
        verify_encryption_group_for_tally_bb_dir(bb, eg, result);
    }
}

#[cfg(test)]
mod test {
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;

    use super::*;
    use crate::{
        config::test::{
            CONFIG_TEST, get_test_verifier_mock_tally_dir,
            get_test_verifier_tally_dir as get_verifier_dir,
        },
        consts::NUMBER_CONTROL_COMPONENTS,
    };

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_encryption_group() {
        let eg_expected = EncryptionParameters::from((
            &Integer::from(10usize),
            &Integer::from(15usize),
            &Integer::from(3usize),
        ));
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(10usize),
            &Integer::from(15usize),
            &Integer::from(3usize),
        ));
        result.append_with_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(result.is_ok());
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(11usize),
            &Integer::from(15usize),
            &Integer::from(3usize),
        ));
        result.append_with_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(!result.has_errors());
        assert_eq!(result.failures().len(), 1);
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(11usize),
            &Integer::from(16usize),
            &Integer::from(4usize),
        ));
        result.append_with_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(!result.has_errors());
        assert_eq!(result.failures().len(), 3)
    }

    #[test]
    fn change_context() {
        // p
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_tally_dir();
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.encryption_group.set_p(&Integer::from(1234usize));
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
        // q
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_tally_dir();
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.encryption_group.set_q(&Integer::from(1234usize));
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
        // g
        let mut result = VerificationResult::new();
        let mut mock_dir = get_test_verifier_mock_tally_dir();
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(|d| {
                d.encryption_group.set_g(&Integer::from(1234usize));
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(!result.has_errors());
        assert!(result.has_failures());
    }

    #[test]
    fn change_tally_shuffle() {
        let nb = get_test_verifier_mock_tally_dir()
            .unwrap_tally()
            .bb_directories()
            .len();
        for i in 0..nb {
            // p
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                .mock_tally_component_shuffle_payload(|d| {
                    d.encryption_group.set_p(&Integer::from(1234usize));
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
            // q
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                .mock_tally_component_shuffle_payload(|d| {
                    d.encryption_group.set_q(&Integer::from(1234usize));
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
            // g
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                .mock_tally_component_shuffle_payload(|d| {
                    d.encryption_group.set_g(&Integer::from(1234usize));
                });
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }

    #[test]
    fn change_tally_votes() {
        let nb = get_test_verifier_mock_tally_dir()
            .unwrap_tally()
            .bb_directories()
            .len();
        for i in 0..nb {
            // p
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i].mock_tally_component_votes_payload(
                |d| {
                    d.encryption_group.set_p(&Integer::from(1234usize));
                },
            );
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
            // q
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i].mock_tally_component_votes_payload(
                |d| {
                    d.encryption_group.set_q(&Integer::from(1234usize));
                },
            );
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
            // g
            let mut result = VerificationResult::new();
            let mut mock_dir = get_test_verifier_mock_tally_dir();
            mock_dir.unwrap_tally_mut().bb_directories_mut()[i].mock_tally_component_votes_payload(
                |d| {
                    d.encryption_group.set_g(&Integer::from(1234usize));
                },
            );
            fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
            assert!(!result.has_errors());
            assert!(result.has_failures());
        }
    }

    #[test]
    fn change_cc_bb() {
        let nb = get_test_verifier_mock_tally_dir()
            .unwrap_tally()
            .bb_directories()
            .len();
        for i in 0..nb {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                // p
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_ballot_box_payload(j, |d| {
                        d.encryption_group.set_p(&Integer::from(1234usize));
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
                // q
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_ballot_box_payload(j, |d| {
                        d.encryption_group.set_q(&Integer::from(1234usize));
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
                // g
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_ballot_box_payload(j, |d| {
                        d.encryption_group.set_g(&Integer::from(1234usize));
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
            }
        }
    }

    #[test]
    fn change_cc_shuffle() {
        let nb = get_test_verifier_mock_tally_dir()
            .unwrap_tally()
            .bb_directories()
            .len();
        for i in 0..nb {
            for j in 1..=NUMBER_CONTROL_COMPONENTS {
                // p
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_shuffle_payload(j, |d| {
                        d.encryption_group.set_p(&Integer::from(1234usize));
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
                // q
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_shuffle_payload(j, |d| {
                        d.encryption_group.set_q(&Integer::from(1234usize));
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
                // g
                let mut result = VerificationResult::new();
                let mut mock_dir = get_test_verifier_mock_tally_dir();
                mock_dir.unwrap_tally_mut().bb_directories_mut()[i]
                    .mock_control_component_shuffle_payload(j, |d| {
                        d.encryption_group.set_g(&Integer::from(1234usize));
                    });
                fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
                assert!(!result.has_errors());
                assert!(result.has_failures());
            }
        }
    }
}
