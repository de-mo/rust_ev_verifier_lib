use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{
        tally_directory::BBDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait,
    },
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();

    for bb_dir in tally_dir.bb_directories().iter() {
        result.append_with_context(
            &verify_for_bb_directory(bb_dir),
            format!("Ballot box directory {}", bb_dir.name()),
        );
    }
}

fn verify_for_bb_directory<B: BBDirectoryTrait>(bb_dir: &B) -> VerificationResult {
    let mut result = VerificationResult::new();

    let bb_name = bb_dir.name();

    let nb_votes = match bb_dir.tally_component_votes_payload() {
        Ok(p) => p.votes.len(),
        Err(e) => {
            result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/tally_component_votes_payload cannot be read",
                bb_name
            )));
            return result;
        }
    };

    for (i, cc_bb_payload_res) in bb_dir.control_component_ballot_box_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => {
                if p.confirmed_encrypted_votes.len() != nb_votes {
                    result.push(VerificationEvent::new_failure(&format!(
                    "The number of vote {} in {}/control_component_shuffle_payload_{} is not the same than the number of votes {} in tally_component_votes_payload",
                    p.confirmed_encrypted_votes.len(), bb_name, i, nb_votes
                )));
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_ballot_box_payload_{} cannot be read",
                bb_name, i
            ))),
        }
    }

    for (i, cc_bb_payload_res) in bb_dir.control_component_shuffle_payload_iter() {
        match cc_bb_payload_res {
            Ok(p) => {
                if nb_votes < 2 {
                    if p.verifiable_shuffle.shuffled_ciphertexts.len() != nb_votes + 2 {
                        result.push(VerificationEvent::new_failure(&format!(
                            "The number of vote {} in {}/control_component_ballot_box_payload_{} must be {}, since the number of votes {} in tally_component_votes_payload is less than 2",
                            p.verifiable_shuffle.shuffled_ciphertexts.len(), bb_name, i, nb_votes + 2, nb_votes
                        )));
                    }
                } else if p.verifiable_shuffle.shuffled_ciphertexts.len() != nb_votes {
                    result.push(VerificationEvent::new_failure(&format!(
                    "The number of vote {} in {}/control_component_ballot_box_payload_{} is not the same than the number of votes {} in tally_component_votes_payload",
                    p.verifiable_shuffle.shuffled_ciphertexts.len(), bb_name, i, nb_votes
                )));
                }
            }
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_shuffle_payload_{} cannot be read",
                bb_name, i
            ))),
        }
    }

    match bb_dir.tally_component_shuffle_payload() {
        Ok(p) => {
            if nb_votes < 2 {
                if p.verifiable_plaintext_decryption.decrypted_votes.len() != nb_votes + 2 {
                    result.push(VerificationEvent::new_failure(&format!(
                        "The number of vote {} in {}/tally_component_shuffle_payload must be {}, since the number of votes {} in tally_component_votes_payload is less than 2",
                        p.verifiable_plaintext_decryption.decrypted_votes.len(), bb_name, nb_votes + 2, nb_votes
                    )));
                }
            } else if p.verifiable_plaintext_decryption.decrypted_votes.len() != nb_votes {
                result.push(VerificationEvent::new_failure(&format!(
                "The number of vote {} in {}/tally_component_shuffle_payload is not the same than the number of votes {} in tally_component_votes_payload",
                p.verifiable_plaintext_decryption.decrypted_votes.len(), bb_name, nb_votes
            )));
            }
        }
        Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
            "{}/tally_component_shuffle_payload cannot be read",
            bb_name
        ))),
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_tally_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for r in result.errors_to_string() {
                println!("{:?}", r)
            }
            for r in result.failures_to_string() {
                println!("{:?}", r)
            }
        }
        assert!(result.is_ok());
    }
}
