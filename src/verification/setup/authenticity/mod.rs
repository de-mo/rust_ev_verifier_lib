use super::super::{
    result::{create_verification_error, VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
    verify_signature_for_object,
};
use crate::{
    config::Config,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::meta_data::VerificationMetaDataList,
};
use anyhow::anyhow;
use log::debug;

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static Config,
) -> anyhow::Result<VerificationList<'a>> {
    Ok(VerificationList(vec![
        Verification::new(
            "02.01",
            "VerifySignatureCantonConfig",
            fn_0201_verify_signature_canton_config,
            metadata_list,
            config,
        )?,
        Verification::new(
            "02.02",
            "VerifySignatureSetupComponentPublicKeys",
            fn_0202_verify_signature_setup_component_public_keys,
            metadata_list,
            config,
        )?,
        Verification::new(
            "02.03",
            "VerifySignatureControlComponentPublicKeys",
            fn_0203_verify_signature_control_component_public_keys,
            metadata_list,
            config,
        )?,
        Verification::new(
            "02.04",
            "VerifySignatureSetupComponentTallyData",
            fn_0204_verify_signature_setup_component_tally_data,
            metadata_list,
            config,
        )?,
        Verification::new(
            "02.05",
            "VerifySignatureElectionEventContext",
            fn_0205_verify_signature_election_event_context,
            metadata_list,
            config,
        )?,
    ]))
}

fn fn_0201_verify_signature_canton_config<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let ee_config = match context_dir.election_event_configuration() {
        Ok(p) => p,
        Err(e) => {
            result.push(create_verification_error!(
                format!("{} cannot be read", "election_event_configuration"),
                e
            ));
            return;
        }
    };
    verify_signature_for_object(
        ee_config.as_ref(),
        result,
        config,
        "election_event_configuration",
    )
}

fn fn_0202_verify_signature_setup_component_public_keys<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let eg = match context_dir.setup_component_public_keys_payload() {
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
        config,
        "setup_component_public_keys_payload",
    )
}

fn fn_0203_verify_signature_control_component_public_keys<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    for (i, cc) in context_dir.control_component_public_keys_payload_iter() {
        debug!("Verification 2.03 for cc {}", i);
        match cc {
            Ok(cc) => verify_signature_for_object(
                cc.as_ref(),
                result,
                config,
                &format!("control_component_public_keys_payload_{}", i),
            ),
            Err(e) => result.push(create_verification_error!(
                format!("control_component_public_keys_payload_{} cannot be read", i),
                e
            )),
        }
    }
}

fn fn_0204_verify_signature_setup_component_tally_data<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    for d in context_dir.vcs_directories() {
        debug!("Verification 2.04 for vcs_dir {}", d.get_name());
        match d.setup_component_tally_data_payload() {
            Ok(p) => verify_signature_for_object(
                p.as_ref(),
                result,
                config,
                &format!("{}/setup_component_tally_data_payload.json", d.get_name(),),
            ),
            Err(e) => result.push(create_verification_error!(
                format!("{}/setup_component_tally_data_payload.json", d.get_name(),),
                e
            )),
        }
    }
}

fn fn_0205_verify_signature_election_event_context<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let rp = match context_dir.election_event_context_payload() {
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
        config,
        "election_event_context_payload",
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    #[ignore = "error with XML"]
    fn test_0201() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0201_verify_signature_canton_config(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_0202() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0202_verify_signature_setup_component_public_keys(&dir, &CONFIG_TEST, &mut result);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_0203() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0203_verify_signature_control_component_public_keys(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_0204() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0204_verify_signature_setup_component_tally_data(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_0205() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0205_verify_signature_election_event_context(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
