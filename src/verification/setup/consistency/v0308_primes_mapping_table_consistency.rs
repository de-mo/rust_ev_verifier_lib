use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};

use std::collections::HashMap;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let ee_c_paylod = match context_dir.election_event_context_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot extract election_event_context_payload"),
            );
            return;
        }
    };

    let mut primes_hashmaps: HashMap<usize, String> = HashMap::new();
    for ee_context in ee_c_paylod
        .election_event_context
        .verification_card_set_contexts
    {
        for p_table in ee_context.primes_mapping_table.p_table {
            match primes_hashmaps.get(&p_table.encoded_voting_option) {
                Some(option) => {
                    if option != &p_table.actual_voting_option {
                        result.push(VerificationEvent::new_failure(&format!(
                            "The prime {} encode two different voting options {} and {}",
                            p_table.encoded_voting_option, p_table.actual_voting_option, option
                        )));
                    }
                }
                None => {
                    let _ = primes_hashmaps.insert(
                        p_table.encoded_voting_option,
                        p_table.actual_voting_option.clone(),
                    );
                }
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    #[ignore = "Implementation to be changed"]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
