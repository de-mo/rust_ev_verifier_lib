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

fn verrify_card_set_ids_setup_vcs<V: SetupVCSDirectoryTrait>(vcs_dir: &V) -> VerificationResult {
    let mut res = VerificationResult::new();
    let vcs_id = vcs_dir.get_name();
    for (chunk, payload_res) in vcs_dir.setup_component_verification_data_payload_iter() {
        if let Err(e) = payload_res {
            res.push(create_verification_error!(
                format!(
                    "Cannot read payload for setup_component_verification_data.{}",
                    chunk
                ),
                e
            ));
            break;
        }
        if payload_res.unwrap().verification_card_set_id != vcs_id {
            res.push(
                create_verification_failure!(
                    format!(
                        "verification card set in file setup_component_verification_data.{} doesn't match with expected {}",
                        chunk,
                        vcs_id
                    )
                )
            );
        }
    }

    for (chunk, payload_res) in vcs_dir.control_component_code_shares_payload_iter() {
        if let Err(e) = payload_res {
            res.push(create_verification_error!(
                format!(
                    "Cannot read payload for setup_component_verification_data.{}",
                    chunk
                ),
                e
            ));
            break;
        }
        for payload in payload_res.unwrap().0.iter() {
            if payload.verification_card_set_id != vcs_id {
                res.push(
                    create_verification_failure!(
                        format!(
                            "verification card set for node {} in file setup_component_verification_data.{} doesn't match with expected {}",
                            payload.node_id,
                            chunk,
                            vcs_id
                        )
                    )
                );
            }
        }
    }
    res
}

fn verrify_card_set_ids_context_vcs<V: ContextVCSDirectoryTrait>(
    vcs_dir: &V,
) -> VerificationResult {
    let mut res = VerificationResult::new();
    let vcs_id = vcs_dir.get_name();
    match vcs_dir.setup_component_tally_data_payload() {
        Ok(p) => {
            if p.verification_card_set_id != vcs_id {
                res.push(
                create_verification_failure!(
                    format!(
                        "verification card set in file setup_component_tally_data_payload doesn't match with expected {}",
                        vcs_id
                    )
                )
            );
            }
        }
        Err(e) => res.push(create_verification_error!(
            "Cannot read payload for setup_component_tally_data_payload",
            e
        )),
    }
    res
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_dir = dir.unwrap_setup();

    for vcs_dir in context_dir.vcs_directories().iter() {
        result.append_wtih_context(
            &verrify_card_set_ids_context_vcs(vcs_dir),
            format!("context vcs directory {}", vcs_dir.get_name()),
        );
    }

    for vcs_dir in setup_dir.vcs_directories().iter() {
        result.append_wtih_context(
            &verrify_card_set_ids_setup_vcs(vcs_dir),
            format!("setup vcs directory {}", vcs_dir.get_name()),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
