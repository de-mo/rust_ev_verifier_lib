use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use rust_ev_crypto_primitives::elgamal::EncryptionParameters;

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

fn verify_encryption_group_for_context_vcs_dir<V: ContextVCSDirectoryTrait>(
    dir: &V,
    eg: &EncryptionParameters,
    result: &mut VerificationResult,
) {
    match dir.setup_component_tally_data_payload() {
        Ok(p) => result.append_with_context(
            &verify_encryption_group(&p.encryption_group, eg),
            format!("{}/setup_component_tally_data_payload", dir.name()),
        ),
        Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
            "{}/setup_component_tally_data_payload has wrong format",
            dir.name()
        ))),
    }
}

fn verify_encryption_group_for_setup_vcs_dir<V: SetupVCSDirectoryTrait>(
    dir: &V,
    eg: &EncryptionParameters,
    result: &mut VerificationResult,
) {
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        match f {
            Ok(s) => result.append_with_context(
                &verify_encryption_group(&s.encryption_group, eg),
                format!(
                    "{}/setup_component_verification_data_payload.{}",
                    i,
                    dir.name()
                ),
            ),
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/setup_component_verification_data_payload.{} has wrong format",
                dir.name(),
                i
            ))),
        }
    }
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        match f {
            Ok(cc) => {
                for (j, p) in cc.0.iter().enumerate() {
                    result.append_with_context(
                        &verify_encryption_group(&p.encryption_group, eg),
                        format!(
                            "{}/control_component_code_shares_payload.{}_chunk{}_element{}",
                            dir.name(),
                            i,
                            p.chunk_id,
                            j
                        ),
                    )
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_code_shares_payload_.{} has wrong format",
                dir.name(),
                i
            ))),
        }
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let config_dir = dir.context();
    let setup_dir = dir.unwrap_setup();
    let eg = match config_dir.election_event_context_payload() {
        Ok(p) => p.encryption_group,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    for (i, f) in config_dir.control_component_public_keys_payload_iter() {
        match f {
            Ok(cc) => result.append_with_context(
                &verify_encryption_group(&cc.encryption_group, &eg),
                format!("control_component_public_keys_payload.{}", i),
            ),
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "control_component_public_keys_payload.{} has wrong format",
                i
            ))),
        }
    }
    match config_dir.setup_component_public_keys_payload() {
        Ok(p) => result.append_with_context(
            &verify_encryption_group(&p.encryption_group, &eg),
            "setup_component_public_keys_payload",
        ),
        Err(e) => result.push(
            VerificationEvent::new_error(&e)
                .add_context("election_event_context_payload has wrong format"),
        ),
    }

    for vcs in config_dir.vcs_directories().iter() {
        verify_encryption_group_for_context_vcs_dir(vcs, &eg, result);
    }

    for vcs in setup_dir.vcs_directories().iter() {
        verify_encryption_group_for_setup_vcs_dir(vcs, &eg, result);
    }
}

#[cfg(test)]
mod test {
    use rust_ev_crypto_primitives::Integer;

    use super::*;
    use crate::config::test::{
        get_test_verifier_mock_setup_dir as get_mock_verifier_dir,
        get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST,
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
    fn test_wrong_election_event_context() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_mock_verifier_dir();
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
        mock_dir
            .context_mut()
            .mock_control_component_public_keys_payload(2, |d| {
                d.encryption_group.set_p(&Integer::from(1234usize));
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }

    #[test]
    fn test_wrong_control_component_public_keys() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_mock_verifier_dir();
        mock_dir
            .context_mut()
            .mock_control_component_public_keys_payload(2, |d| {
                d.encryption_group.set_p(&Integer::from(1234usize));
                d.encryption_group.set_q(&Integer::from(1234usize))
            });
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }
}
