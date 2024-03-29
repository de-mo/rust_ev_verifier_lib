use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    file_structure::{
        setup_directory::{SetupDirectoryTrait, VCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use anyhow::anyhow;
use log::debug;
use rust_ev_crypto_primitives::EncryptionParameters;

fn verify_encryption_group(
    eg: &EncryptionParameters,
    expected: &EncryptionParameters,
    name: &str,
    result: &mut VerificationResult,
) {
    if eg.p() != expected.p() {
        result.push(create_verification_failure!(format!(
            "p not equal in {}",
            name
        )));
    }
    if eg.q() != expected.q() {
        result.push(create_verification_failure!(format!(
            "q not equal in {}",
            name
        )));
    }
    if eg.g() != expected.g() {
        result.push(create_verification_failure!(format!(
            "g not equal in {}",
            name
        )));
    }
}

fn verify_encryption_group_for_vcs_dir<V: VCSDirectoryTrait>(
    dir: &V,
    eg: &EncryptionParameters,
    result: &mut VerificationResult,
) {
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        match f {
            Ok(s) => verify_encryption_group(
                &s.encryption_group,
                eg,
                &format!(
                    "{}/setup_component_verification_data_payload.{}",
                    i,
                    dir.get_name()
                ),
                result,
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
                for (j, p) in cc.iter().enumerate() {
                    verify_encryption_group(
                        &p.encryption_group,
                        eg,
                        &format!(
                            "{}/control_component_code_shares_payload.{}_chunk{}_element{}",
                            dir.get_name(),
                            i,
                            p.chunk_id,
                            j
                        ),
                        result,
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
    match dir.setup_component_tally_data_payload() {
        Ok(p) => verify_encryption_group(
            &p.encryption_group,
            eg,
            &format!("{}/setup_component_tally_data_payload", dir.get_name()),
            result,
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

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.election_event_context_payload() {
        Ok(p) => p.encryption_group,
        Err(e) => {
            result.push(create_verification_error!(
                "election_event_context_payload cannot be read",
                e
            ));
            return;
        }
    };
    for (i, f) in setup_dir.control_component_public_keys_payload_iter() {
        match f {
            Ok(cc) => verify_encryption_group(
                &cc.encryption_group,
                &eg,
                &format!("control_component_public_keys_payload.{}", i),
                result,
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
    match setup_dir.setup_component_public_keys_payload() {
        Ok(p) => verify_encryption_group(
            &p.encryption_group,
            &eg,
            "setup_component_public_keys_payload",
            result,
        ),
        Err(e) => result.push(create_verification_error!(
            "election_event_context_payload has wrong format",
            e
        )),
    }
    for vcs in setup_dir.vcs_directories().iter() {
        verify_encryption_group_for_vcs_dir(vcs, &eg, result);
    }
}

#[cfg(test)]
mod test {
    use rug::Integer;

    use super::{
        super::super::super::{result::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::config::test::{
        get_test_verifier_setup_dir as get_verifier_dir, test_dataset_setup_path, CONFIG_TEST,
    };
    use crate::{
        data_structures::VerifierSetupDataTrait, file_structure::mock::MockVerificationDirectory,
    };

    fn get_mock_verifier_dir() -> MockVerificationDirectory {
        MockVerificationDirectory::new(&VerificationPeriod::Setup, &test_dataset_setup_path())
    }

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
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
        verify_encryption_group(&eg, &eg_expected, "toto", &mut result);
        assert!(result.is_ok().unwrap());
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(11usize),
            &Integer::from(15usize),
            &Integer::from(3usize),
        ));
        verify_encryption_group(&eg, &eg_expected, "toto", &mut result);
        assert!(!result.has_errors().unwrap());
        assert_eq!(result.failures().len(), 1);
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(11usize),
            &Integer::from(16usize),
            &Integer::from(4usize),
        ));
        verify_encryption_group(&eg, &eg_expected, "toto", &mut result);
        assert!(!result.has_errors().unwrap());
        assert_eq!(result.failures().len(), 3)
    }

    #[test]
    fn test_wrong_election_event_context() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_mock_verifier_dir();
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
        let mut eec = mock_dir
            .unwrap_setup()
            .election_event_context_payload()
            .unwrap();
        eec.encryption_group.set_p(&Integer::from(1234usize));
        mock_dir
            .unwrap_setup_mut()
            .mock_election_event_context_payload(&Ok(&eec));
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures().unwrap());
    }

    #[test]
    fn test_wrong_control_component_public_keys() {
        let mut result = VerificationResult::new();
        let mut mock_dir = get_mock_verifier_dir();
        let mut cc_pk = mock_dir
            .unwrap_setup()
            .control_component_public_keys_payload_group()
            .get_file_with_number(2)
            .get_data()
            .map(|d| Box::new(d.control_component_public_keys_payload().unwrap().clone()))
            .unwrap();
        cc_pk.encryption_group.set_p(&Integer::from(1234usize));
        cc_pk.encryption_group.set_q(&Integer::from(1234usize));
        mock_dir
            .unwrap_setup_mut()
            .mock_control_component_public_keys_payloads(2, &Ok(&cc_pk));
        fn_verification(&mock_dir, &CONFIG_TEST, &mut result);
        assert!(result.has_failures().unwrap());
    }
}
