use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    file_structure::{
        tally_directory::BBDirectoryTrait, ContextDirectoryTrait, TallyDirectoryTrait,
        VerificationDirectoryTrait,
    },
};
use rust_ev_system_library::preliminaries::PTableTrait;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let tally_dir = dir.unwrap_tally();

    let vc_contexts = match context_dir.election_event_context_payload() {
        Ok(p) => p.election_event_context.verification_card_set_contexts,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };

    let mut res = VerificationResult::join(
        tally_dir
            .bb_directories()
            .iter()
            .map(
                |dir| match vc_contexts.iter().find(|c| c.ballot_box_id == dir.name()) {
                    Some(c) => {
                        verify_pro_ballot_box(dir, c.primes_mapping_table.p_table.get_delta())
                            .clone_add_context(format!("ballot box {}", dir.name()))
                    }
                    None => VerificationResult::from(&VerificationEvent::new_error(&format!(
                        "context for ballot box id {} not found",
                        dir.name()
                    ))),
                },
            )
            .collect::<Vec<_>>()
            .as_slice(),
    );
    result.append(&mut res);
}

fn verify_pro_ballot_box<B: BBDirectoryTrait>(bb_dir: &B, delta: usize) -> VerificationResult {
    let mut res = VerificationResult::new();

    let tally_shuffle_payload = match bb_dir.tally_component_shuffle_payload() {
        Ok(tally_shuffle_payload) => tally_shuffle_payload,
        Err(e) => {
            return VerificationResult::from(&VerificationEvent::new_error(&e).add_context(
                format!(
                    "{}/tally_component_shuffle_payload has wrong format",
                    bb_dir.name(),
                ),
            ))
        }
    };

    res.append(&mut VerificationResult::from(
        tally_shuffle_payload
            .verifiable_plaintext_decryption
            .decrypted_votes
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if v.message.len() == delta {
                    None
                } else {
                    Some(VerificationEvent::new_failure(&format!(
                    "size of message of decrypted vote at pos {} is not the same as delta + 1 = {}",
                    i,
                    delta + 1
                )))
                }
            })
            .collect::<Vec<_>>()
            .as_slice(),
    ));

    res.append(&mut VerificationResult::from(
        tally_shuffle_payload
            .verifiable_plaintext_decryption
            .decryption_proofs
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.z.len() == delta {
                    None
                } else {
                    Some(VerificationEvent::new_failure(&format!(
                        "size of message of proofs at pos {} is not the same as delta + 1 = {}",
                        i,
                        delta + 1
                    )))
                }
            })
            .collect::<Vec<_>>()
            .as_slice(),
    ));
    res
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
