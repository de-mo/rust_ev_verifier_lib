use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use anyhow::anyhow;
use log::debug;
use rust_ev_crypto_primitives::EncryptionParameters;

fn verify_encryption_group(
    eg: &EncryptionParameters,
    expected: &EncryptionParameters,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    if eg.p() != expected.p() {
        result.push(create_verification_failure!(format!("p not equal",)));
    }
    if eg.q() != expected.q() {
        result.push(create_verification_failure!(format!("q not equal",)));
    }
    if eg.g() != expected.g() {
        result.push(create_verification_failure!(format!("g not equal",)));
    }
    result
}

fn verify_encryption_group_for_context_vcs_dir<V: ContextVCSDirectoryTrait>(
    dir: &V,
    eg: &EncryptionParameters,
    result: &mut VerificationResult,
) {
    match dir.setup_component_tally_data_payload() {
        Ok(p) => result.append_wtih_context(
            &verify_encryption_group(&p.encryption_group, eg),
            format!("{}/setup_component_tally_data_payload", dir.get_name()),
        ),
        Err(e) => result.push(create_verification_error!(
            format!(
                "{}/setup_component_tally_data_payload has wrong format",
                dir.get_name()
            ),
            e
        )),
    }
}

fn verify_encryption_group_for_setup_vcs_dir<V: SetupVCSDirectoryTrait>(
    dir: &V,
    eg: &EncryptionParameters,
    result: &mut VerificationResult,
) {
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        match f {
            Ok(s) => result.append_wtih_context(
                &verify_encryption_group(&s.encryption_group, eg),
                format!(
                    "{}/setup_component_verification_data_payload.{}",
                    i,
                    dir.get_name()
                ),
            ),
            Err(e) => result.push(create_verification_error!(
                format!(
                    "{}/setup_component_verification_data_payload.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                e
            )),
        }
    }
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        match f {
            Ok(cc) => {
                for (j, p) in cc.0.iter().enumerate() {
                    result.append_wtih_context(
                        &verify_encryption_group(&p.encryption_group, eg),
                        format!(
                            "{}/control_component_code_shares_payload.{}_chunk{}_element{}",
                            dir.get_name(),
                            i,
                            p.chunk_id,
                            j
                        ),
                    )
                }
            }
            Err(e) => result.push(create_verification_error!(
                format!(
                    "{}/control_component_code_shares_payload_.{} has wrong format",
                    dir.get_name(),
                    i
                ),
                e
            )),
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
            result.push(create_verification_error!(
                "election_event_context_payload cannot be read",
                e
            ));
            return;
        }
    };
    for (i, f) in config_dir.control_component_public_keys_payload_iter() {
        match f {
            Ok(cc) => result.append_wtih_context(
                &verify_encryption_group(&cc.encryption_group, &eg),
                format!("control_component_public_keys_payload.{}", i),
            ),
            Err(e) => result.push(create_verification_error!(
                format!(
                    "control_component_public_keys_payload.{} has wrong format",
                    i
                ),
                e
            )),
        }
    }
    match config_dir.setup_component_public_keys_payload() {
        Ok(p) => result.append_wtih_context(
            &verify_encryption_group(&p.encryption_group, &eg),
            "setup_component_public_keys_payload",
        ),
        Err(e) => result.push(create_verification_error!(
            "election_event_context_payload has wrong format",
            e
        )),
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
    use crate::data_structures::VerifierContextDataTrait;

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
        result.append_wtih_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(result.is_ok());
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(11usize),
            &Integer::from(15usize),
            &Integer::from(3usize),
        ));
        result.append_wtih_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(!result.has_errors());
        assert_eq!(result.failures().len(), 1);
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(11usize),
            &Integer::from(16usize),
            &Integer::from(4usize),
        ));
        result.append_wtih_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(!result.has_errors());
        assert_eq!(result.failures().len(), 3)
    }

    #[test]
    fn test_wrong_election_event_context() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_mock_verifier_dir();
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
        let mut eec = mock_dir.context().election_event_context_payload().unwrap();
        eec.encryption_group.set_p(&Integer::from(1234usize));
        mock_dir
            .context_mut()
            .mock_election_event_context_payload(&Ok(&eec));
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }

    #[test]
    fn test_wrong_control_component_public_keys() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_mock_verifier_dir();
        let mut cc_pk = mock_dir
            .context()
            .control_component_public_keys_payload_group()
            .get_file_with_number(2)
            .get_data()
            .map(|d| Box::new(d.control_component_public_keys_payload().unwrap().clone()))
            .unwrap();
        cc_pk.encryption_group.set_p(&Integer::from(1234usize));
        cc_pk.encryption_group.set_q(&Integer::from(1234usize));
        mock_dir
            .context_mut()
            .mock_control_component_public_keys_payloads(2, &Ok(&cc_pk));
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures());
    }
}
