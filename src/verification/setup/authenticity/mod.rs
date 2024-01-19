use super::super::{
    result::{create_verification_error, VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
    verify_signature_for_object,
};
use crate::{
    config::Config,
    file_structure::{
        setup_directory::{SetupDirectoryTrait, VCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::meta_data::VerificationMetaDataList,
};
use anyhow::anyhow;
use log::debug;

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> VerificationList<'a> {
    VerificationList(vec![
        Verification::new("02.01", fn_verification_0201, metadata_list, config).unwrap(),
        Verification::new("02.03", fn_verification_0203, metadata_list, config).unwrap(),
        Verification::new("02.04", fn_verification_0204, metadata_list, config).unwrap(),
        Verification::new("02.05", fn_verification_0205, metadata_list, config).unwrap(),
        Verification::new("02.06", fn_verification_0206, metadata_list, config).unwrap(),
        Verification::new("02.07", fn_verification_0207, metadata_list, config).unwrap(),
    ])
}

fn fn_verification_0201<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(create_verification_error!(
                format!("{} cannot be read", "encryption_parameters_payload"),
                e
            ));
            return;
        }
    };
    verify_signature_for_object(
        eg.as_ref(),
        result,
        config.keystore(),
        "encryption_parameters_payload",
    )
}

fn fn_verification_0203<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.setup_component_public_keys_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(create_verification_error!(
                format!("{} cannot be read", "setup_component_public_keys_payload"),
                e
            ));
            return;
        }
    };
    verify_signature_for_object(
        eg.as_ref(),
        result,
        config.keystore(),
        "setup_component_public_keys_payload",
    )
}

fn fn_verification_0204<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    for (i, cc) in setup_dir.control_component_public_keys_payload_iter() {
        debug!("Verification 2.04 for cc {}", i);
        match cc {
            Ok(cc) => verify_signature_for_object(
                cc.as_ref(),
                result,
                config.keystore(),
                &format!("control_component_public_keys_payload_{}", i),
            ),
            Err(e) => result.push(create_verification_error!(
                format!("control_component_public_keys_payload_{} cannot be read", i),
                e
            )),
        }
    }
}

fn fn_verification_0205<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    for d in setup_dir.vcs_directories() {
        debug!("Verification 2.05 for vcs_dir {}", d.get_name());
        for (i, ps) in d.setup_component_verification_data_payload_iter() {
            match ps {
                Ok(p) => verify_signature_for_object(
                    p.as_ref(),
                    result,
                    config.keystore(),
                    &format!(
                        "{}/setup_component_verification_data_payload_iter.{}.json",
                        d.get_name(),
                        i
                    ),
                ),
                Err(e) => result.push(create_verification_error!(
                    format!(
                        "{}/setup_component_verification_data_payload_iter.{}.json",
                        d.get_name(),
                        i
                    ),
                    e
                )),
            }
        }
    }
}

fn fn_verification_0206<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    for d in setup_dir.vcs_directories() {
        debug!("Verification 2.06 for vcs_dir {}", d.get_name());
        for (i, rps) in d.control_component_code_shares_payload_iter() {
            match rps {
                Ok(ps) => {
                    for p in ps.iter() {
                        verify_signature_for_object(
                            p,
                            result,
                            config.keystore(),
                            &format!(
                                "{}/control_component_code_shares_payload.{}.json[{}]",
                                d.get_name(),
                                i,
                                p.node_id
                            ),
                        )
                    }
                }
                Err(e) => result.push(create_verification_error!(
                    format!(
                        "Error reading {}/control_component_code_shares_payload.{}.json",
                        d.get_name(),
                        i
                    ),
                    e
                )),
            }
        }
    }
}

fn fn_verification_0207<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    for d in setup_dir.vcs_directories() {
        debug!("Verification 2.07 for vcs_dir {}", d.get_name());
        match d.setup_component_tally_data_payload() {
            Ok(p) => verify_signature_for_object(
                p.as_ref(),
                result,
                config.keystore(),
                &format!("{}/setup_component_tally_data_payload.json", d.get_name(),),
            ),
            Err(e) => result.push(create_verification_error!(
                format!("{}/setup_component_tally_data_payload.json", d.get_name(),),
                e
            )),
        }
    }
}

#[allow(dead_code)]
fn fn_verification_0208<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let rp = match setup_dir.election_event_context_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(create_verification_error!(
                format!("{} cannot be read", "election_event_context_payload"),
                e
            ));
            return;
        }
    };
    verify_signature_for_object(
        rp.as_ref(),
        result,
        config.keystore(),
        "election_event_context_payload",
    )
}

#[cfg(test)]
mod test {
    use super::{super::super::result::VerificationResultTrait, *};
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_0201() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0201(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0203() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0203(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0204() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0204(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0205() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0205(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0206() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0206(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    fn test_0207() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0207(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }

    #[test]
    #[ignore]
    fn test_0208() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification_0208(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
