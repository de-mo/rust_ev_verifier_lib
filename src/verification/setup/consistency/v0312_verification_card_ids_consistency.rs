// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::collections::{HashMap, HashSet};

use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
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
    let payload = match vcs_dir.setup_component_tally_data_payload() {
        Ok(p) => p,
        Err(e) => {
            res.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context("Cannot read payload for setup_component_tally_data_payload"),
            );
            return (vec![], res);
        }
    };
    let vc_ids = &payload.verification_card_ids;
    let mut uniq = HashSet::new();
    let no_duplicate = vc_ids.iter().all(move |x| uniq.insert(x));
    if !no_duplicate {
        res.push(VerificationEvent::new_failure(&format!(
            "The list of vc_ids [{}] are not unique in setup_component_tally_data_payload",
            vc_ids.join(",")
        )));
    }
    (vc_ids.clone(), res)
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();

    let mut hm_vc_ids = HashMap::new();

    for vcs_dir in context_dir.vcs_directories().iter() {
        let (vc_ids, res) = verrify_card_ids_context_vcs(vcs_dir);
        result.append_with_context(&res, format!("context vcs directory {}", vcs_dir.name()));
        hm_vc_ids.insert(vcs_dir.name(), vc_ids);
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
