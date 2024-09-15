use super::super::{
    result::{
        VerificationEvent, VerificationResult,
    },
    suite::VerificationList,
    verifications::Verification,
};
use crate::{
    config::Config,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::{meta_data::VerificationMetaDataList, VerificationError},
};
use rust_ev_crypto_primitives::VerifyDomainTrait;

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![Verification::new(
        "04.01",
        "VerifySetupIntegrity",
        fn_0401_verify_setup_integrity,
        metadata_list,
        config,
    )?]))
}

fn validate_context_vcs_dir<V: ContextVCSDirectoryTrait>(dir: &V, result: &mut VerificationResult) {
    match dir.setup_component_tally_data_payload() {
        Ok(d) => {
            for e in d.verifiy_domain() {
                result.push(VerificationEvent::new_failure(&e).add_context(format!(
                    "Error verifying domain for {}/setup_component_tally_data_payload",
                    dir.get_name()
                )))
            }
        }
        Err(e) => result.push(VerificationEvent::new_failure(&e).add_context(format!(
            "{}/setup_component_tally_data_payload has wrong format",
            dir.get_name()
        ))),
    }
}

fn validate_setup_vcs_dir<V: SetupVCSDirectoryTrait>(dir: &V, result: &mut VerificationResult) {
    for (i, f) in dir.control_component_code_shares_payload_iter() {
        match f {
            Ok(d) => {
                for e in d.verifiy_domain() {
                    result.push(VerificationEvent::new_failure(&e).add_context(format!(
                        "Error verifying domain for {}/control_component_public_keys_payload.{}",
                        dir.get_name(),
                        i
                    )))
                }
            }
            Err(e) => result.push(VerificationEvent::new_failure(&e)
            .add_context(
                format!(
                    "{}/control_component_code_shares_payload.{} has wrong format",
                    dir.get_name(),
                    i
                )                
            )),
        }
    }
    for (i, f) in dir.setup_component_verification_data_payload_iter() {
        match f {
            Ok(d) => {
                for e in d.verifiy_domain() {
                    result.push(VerificationEvent::new_failure(&e)
                    .add_context(
                        format!(
                            "Error verifying domain for {}/setup_component_verification_data_payload.{}",
                            dir.get_name(),
                            i
                        )
                        
                    ))
                }
            }
            Err(e) => result.push(VerificationEvent::new_failure(&e)
            .add_context(
                format!(
                    "{}/setup_component_verification_data_payload.{} has wrong format",
                    dir.get_name(),
                    i
                )
                
            )),
        }
    }
}

fn validate_setup_dir<S: SetupDirectoryTrait>(dir: &S, result: &mut VerificationResult) {
    for d in dir.vcs_directories().iter() {
        validate_setup_vcs_dir(d, result);
    }
}

fn validate_context_dir<C: ContextDirectoryTrait>(dir: &C, result: &mut VerificationResult) {
    match dir.election_event_context_payload() {
        Ok(d) => {
            for e in d.verifiy_domain() {
                result.push(VerificationEvent::new_failure(&e)
                .add_context(
                    "Error verifying domain for election_event_context_payload"
                    
                ))
            }
        }
        Err(e) => result.push(VerificationEvent::new_failure(&e)
        .add_context(
            "election_event_context_payload has wrong format"            
        )),
    }
    match dir.setup_component_public_keys_payload() {
        Ok(d) => {
            for e in d.verifiy_domain() {
                result.push(VerificationEvent::new_failure(&e)
                .add_context(
                    "Error verifying domain for setup_component_public_keys_payload"
                    
                ))
            }
        }
        Err(e) => result.push(VerificationEvent::new_failure(&e)
        .add_context(
            "setup_component_public_keys_payload has wrong format"
            
        )),
    }
    match dir.election_event_configuration() {
        Ok(d) => {
            for e in d.verifiy_domain() {
                result.push(VerificationEvent::new_failure(&e)
                .add_context(
                    "Error verifying domain for election_event_configuration"
                    
                ))
            }
        }
        Err(e) => result.push(VerificationEvent::new_failure(&e)
        .add_context(
            "election_event_configuration has wrong format"
            
        )),
    }
    for (i, f) in dir.control_component_public_keys_payload_iter() {
        match f {
            Ok(d) => {
                for e in d.verifiy_domain() {
                    result.push(VerificationEvent::new_failure(&e)
                    .add_context(
                        format!(
                            "Error verifying domain for control_component_public_keys_payload.{}",
                            i
                        )
                        
                    ))
                }
            }
            Err(e) => result.push(VerificationEvent::new_failure(&e)
            .add_context(
                format!(
                    "control_component_public_keys_payload.{} has wrong format",
                    i
                )
                
            )),
        }
    }
    for d in dir.vcs_directories().iter() {
        validate_context_vcs_dir(d, result);
    }
}

fn fn_0401_verify_setup_integrity<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    validate_context_dir(context_dir, result);
    let setup_dir = dir.unwrap_setup();
    validate_setup_dir(setup_dir, result);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0401_verify_setup_integrity(&dir, &CONFIG_TEST, &mut result);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
