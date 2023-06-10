use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::file_structure::{setup_directory::SetupDirectoryTrait, VerificationDirectoryTrait};
use anyhow::anyhow;
use log::debug;
use std::collections::HashMap;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let ee_c_paylod = match setup_dir.election_event_context_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(create_verification_error!(
                "Cannot extract election_event_context_payload",
                e
            ));
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
                        result.push(create_verification_failure!(format!(
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
    use super::{super::super::super::result::VerificationResultTrait, *};
    use crate::constants::test::get_verifier_setup_dir as get_verifier_dir;

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
