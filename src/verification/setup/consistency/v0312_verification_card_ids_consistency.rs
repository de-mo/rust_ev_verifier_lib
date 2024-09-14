use std::collections::{HashMap, HashSet};

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
};

fn verify_ids_same(vc_ids: &[String], expected: &[String]) -> VerificationResult {
    let mut res = VerificationResult::new();
    if vc_ids != expected {
        res.push(VerificationEvent::new_failure(&format!(
            "The voting card ids [{}] are not equal to the expected list of voting card ids [{}]",
            vc_ids.join(","),
            expected.join(",")
        )))
    }
    res
}

fn verrify_card_ids_context_vcs<V: ContextVCSDirectoryTrait>(
    vcs_dir: &V,
) -> (Vec<String>, VerificationResult) {
    let mut res = VerificationResult::new();
    let vc_ids = match vcs_dir.setup_component_tally_data_payload() {
        Ok(p) => p.verification_card_ids,
        Err(e) => {
            res.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot read payload for setup_component_tally_data_payload"),
            );
            return (vec![], res);
        }
    };
    let mut uniq = HashSet::new();
    let no_duplicate = vc_ids.iter().all(move |x| uniq.insert(x));
    if !no_duplicate {
        res.push(VerificationEvent::new_failure(&format!(
            "The list of vc_ids [{}] are not unique in setup_component_tally_data_payload",
            vc_ids.join(",")
        )));
    }
    (vc_ids, res)
}

fn verrify_card_ids_setup_vcs<V: SetupVCSDirectoryTrait>(
    vcs_dir: &V,
    expected: &[String],
) -> VerificationResult {
    let mut res = VerificationResult::new();
    let mut hm_vc_ids = HashMap::new();
    hm_vc_ids.insert(1, vec![]);
    hm_vc_ids.insert(2, vec![]);
    hm_vc_ids.insert(3, vec![]);
    hm_vc_ids.insert(4, vec![]);
    for (i, p) in vcs_dir.control_component_code_shares_payload_iter() {
        match p {
            Err(e) => res.push(VerificationEvent::new_error(&e).add_context(format!(
                "Cannot read payload for control_component_code_shares_payload.{}",
                i
            ))),
            Ok(p) => {
                for share in p.0 {
                    let mut v: Vec<String> = share.vc_ids().iter().map(|s| s.to_string()).collect();
                    hm_vc_ids.get_mut(&share.node_id).unwrap().append(&mut v);
                }
            }
        }
    }
    for (node_id, ids) in hm_vc_ids.iter() {
        res.append_with_context(
            &verify_ids_same(ids, expected),
            format!("control_component_code_shares_payload for node {node_id}"),
        );
    }
    let mut vc_ids = vec![];
    for (i, p) in vcs_dir.setup_component_verification_data_payload_iter() {
        match p {
            Err(e) => res.push(VerificationEvent::new_error(&e).add_context(format!(
                "Cannot read payload for setup_component_verification_data_payload.{}",
                i
            ))),
            Ok(p) => {
                let mut ids: Vec<String> = p.vc_ids().iter().map(|s| s.to_string()).collect();
                vc_ids.append(&mut ids)
            }
        }
    }
    res.append_with_context(
        &verify_ids_same(&vc_ids, expected),
        "setup_component_verification_data_payload",
    );
    res
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_dir = dir.unwrap_setup();

    let mut hm_vc_ids = HashMap::new();

    for vcs_dir in context_dir.vcs_directories().iter() {
        let (vc_ids, res) = verrify_card_ids_context_vcs(vcs_dir);
        result.append_with_context(
            &res,
            format!("context vcs directory {}", vcs_dir.get_name()),
        );
        hm_vc_ids.insert(vcs_dir.get_name(), vc_ids);
    }

    for vcs_dir in setup_dir.vcs_directories().iter() {
        let v = hm_vc_ids.get(&vcs_dir.get_name()).unwrap();
        if !v.is_empty() {
            result.append_with_context(
                &verrify_card_ids_setup_vcs(vcs_dir, v),
                format!("setup vcs directory {}", vcs_dir.get_name()),
            );
        }
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
        if !result.is_ok() {
            for e in result.errors() {
                println!("{}", e);
            }
            for f in result.failures() {
                println!("{}", f);
            }
        }
        assert!(result.is_ok());
    }
}
