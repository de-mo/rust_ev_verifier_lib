use super::super::{
    result::{VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
};
use crate::{
    config::Config,
    file_structure::{
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::{meta_data::VerificationMetaDataList, VerificationError},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::VerifyDomainTrait;

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![Verification::new(
        "09.01",
        "VerifyTallyIntegrity",
        fn_0901_verify_tally_integrity,
        metadata_list,
        config,
    )?]))
}

fn validate_bb_dir<B: BBDirectoryTrait>(dir: &B, result: &mut VerificationResult) {
    match dir.tally_component_votes_payload() {
        Ok(d) => {
            for e in d.verifiy_domain() {
                result.push(VerificationEvent::new_failure(&e)
                .add_context(
                    "Error verifying domain for tally_component_votes_payload"
                    
                ))
            }
        }
        Err(e) => result.push(VerificationEvent::new_failure(&e)
        .add_context(
            "tally_component_votes_payload has wrong format"
            
        )),
    }
    match dir.tally_component_shuffle_payload() {
        Ok(d) => {
            for e in d.verifiy_domain() {
                result.push(VerificationEvent::new_failure(&e)
                .add_context(
                    "Error verifying domain for tally_component_shuffle_payload"
                ))
            }
        }
        Err(e) => result.push(VerificationEvent::new_failure(&e)
        .add_context(
            "tally_component_shuffle_payload has wrong format"
        )),
    }

    for (i, f) in dir.control_component_ballot_box_payload_iter() {
        match f {
            Ok(d) => {
                for e in d.verifiy_domain() {
                    result.push(VerificationEvent::new_failure(&e)
                    .add_context(
                        format!(
                             "Error verifying domain for {}/control_component_ballot_box_payload_iter.{}",
                    dir.name(),
                    i
                        )
                        
                    ))
                }
            }
            Err(e) => result.push(VerificationEvent::new_failure(&e)
            .add_context(
                format!(
                    "{}/control_component_ballot_box_payload_iter.{} has wrong format",
                    dir.name(),
                    i
                )
                
            )),
        }
    }

    for (i, f) in dir.control_component_shuffle_payload_iter() {
        match f {
            Ok(d) => {
                for e in d.verifiy_domain() {
                    result.push(VerificationEvent::new_failure(&e)
                    .add_context(
                        format!(
                             "Error verifying domain for {}/control_component_shuffle_payload_iter.{}",
                    dir.name(),
                    i
                        )
                        
                    ))
                }
            }
            Err(e) => result.push(VerificationEvent::new_failure(&e)
            .add_context(
                format!(
                    "{}/control_component_shuffle_payload_iter.{} has wrong format",
                    dir.name(),
                    i
                )
                
            )),
        }
    }
}

fn fn_0901_verify_tally_integrity<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_tally();
    for d in setup_dir.bb_directories().iter() {
        validate_bb_dir(d, result);
    }
}

#[cfg(test)]
mod test {
    use super::{super::super::result::VerificationResult, *};
    use crate::config::test::{get_test_verifier_tally_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0901_verify_tally_integrity(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
