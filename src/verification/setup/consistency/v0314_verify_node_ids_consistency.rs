use std::collections::HashSet;

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        ContextDirectoryTrait, VerificationDirectoryTrait,
    },
};

const LIST_CC_NUMBER: &[usize] = &[1, 2, 3, 4];

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_dir = dir.unwrap_setup();
    result.append(&mut verify_cc_pk_payload(context_dir));
    for vcs in setup_dir.vcs_directories() {
        result.append_with_context(
            &verify_cc_code_shares(vcs),
            format!("vcs dir: {}", vcs.name()),
        );
    }
}

fn verifiy_one_to_for(list: &[usize]) -> VerificationResult {
    let mut result = VerificationResult::new();
    if list.iter().collect::<HashSet<_>>() != LIST_CC_NUMBER.iter().collect::<HashSet<_>>() {
        result.push(VerificationEvent::new_failure(&format!(
            "The list of node ids (={:?}) does not correspond to the expected list (={:?})",
            list, LIST_CC_NUMBER
        )))
    }
    result
}

fn verify_cc_pk_payload<C: ContextDirectoryTrait>(dir: &C) -> VerificationResult {
    let mut result = verifiy_one_to_for(
        dir.control_component_public_keys_payload_group()
            .get_numbers()
            .as_slice(),
    )
    .clone_add_context("context/conntrolComponentPubllicKeysPayload.{}.json");
    for (i, payload_res) in dir.control_component_public_keys_payload_iter() {
        match payload_res {
            Ok(paylod) => {
                let node_id = paylod.control_component_public_keys.node_id;
                if node_id != i {
                    result.push(VerificationEvent::new_failure(&format!(
                        "The node_id (={}) in the file does not correspond to the nr (={}) of the file",
                        node_id, i
                    )))
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&format!(
                "Cannot open conntrolComponentPubllicKeysPayload.{}.json: {}",
                i, e
            ))),
        }
    }
    result
}

fn verify_cc_code_shares<V: SetupVCSDirectoryTrait>(dir: &V) -> VerificationResult {
    let mut result = VerificationResult::new();
    for (chunk_id, payload_res) in dir.control_component_code_shares_payload_iter() {
        match payload_res {
            Ok(paylod) => {
                result.append_with_context(
                    &verifiy_one_to_for(
                        paylod
                            .0
                            .iter()
                            .map(|p| p.node_id)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    ),
                    format!(
                        "setup/{}/controlComponentCodeSharesPayload.{}.json",
                        dir.name(),
                        chunk_id
                    ),
                );
            }
            Err(e) => result.push(VerificationEvent::new_error(&format!(
                "cannot open setup/{}/controlComponentCodeSharesPayload.{}.json: {}",
                dir.name(),
                chunk_id,
                e
            ))),
        }
    }
    result
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
