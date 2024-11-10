use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    data_structures::{
        tally::control_component_ballot_box_payload::ConfirmedEncryptedVote,
        ControlComponentBallotBoxPayload,
    },
    file_structure::{
        tally_directory::BBDirectoryTrait, TallyDirectoryTrait, VerificationDirectoryTrait,
    },
};
use std::fmt::Display;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    let mut res = VerificationResult::join(
        tally_dir
            .bb_directories()
            .iter()
            .map(verify_pro_ballot_box)
            .collect::<Vec<_>>()
            .as_slice(),
    );
    result.append(&mut res);
}

fn verify_pro_ballot_box<B: BBDirectoryTrait>(bb_dir: &B) -> VerificationResult {
    let context = format!("Ballot box dir {}", bb_dir.name());
    let mut cc_bb_payload_iter = bb_dir.control_component_ballot_box_payload_iter();
    let first_node = match cc_bb_payload_iter.next() {
        Some((i, res)) => match res {
            Ok(r) => r,
            Err(e) => {
                return VerificationResult::from(&VerificationEvent::new_error(&e).add_context(
                    format!(
                        "{}/control_component_ballot_box_payload_.{} has wrong format",
                        bb_dir.name(),
                        i
                    ),
                ))
            }
        },
        None => {
            return VerificationResult::from(
                &VerificationEvent::new_error("No first node found").add_context(context),
            )
        }
    };
    let mut res = VerificationResult::new();
    for (i, node) in cc_bb_payload_iter {
        match node {
            Ok(n) => res.append_with_context(
                &compare_two_nodes(&first_node, &n),
                format!(
                    "Comparing node {} to node {}",
                    first_node.node_id, n.node_id
                ),
            ),
            Err(e) => res.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_ballot_box_payload_.{} has wrong format",
                bb_dir.name(),
                i
            ))),
        }
    }
    res.add_context(context);
    res
}

fn compare_two_nodes(
    first: &ControlComponentBallotBoxPayload,
    second: &ControlComponentBallotBoxPayload,
) -> VerificationResult {
    // Validate that the vc ids are the same for the two nodes
    let mut res = VerificationResult::new();
    let mut first_vc_ids = first
        .confirmed_encrypted_votes
        .iter()
        .map(|v| v.context_ids.verification_card_id.as_str())
        .collect::<Vec<_>>();
    first_vc_ids.sort();
    let len = first_vc_ids.len();
    first_vc_ids.dedup();
    if first_vc_ids.len() != len {
        res.push(VerificationEvent::new_error(
            "The voting card set ids are not unique in the list of confirmed encrypted votes (first node)",
        ));
    }
    let mut second_vc_ids = second
        .confirmed_encrypted_votes
        .iter()
        .map(|v| v.context_ids.verification_card_id.as_str())
        .collect::<Vec<_>>();
    second_vc_ids.sort();
    let len = second_vc_ids.len();
    second_vc_ids.dedup();
    if second_vc_ids.len() != len {
        res.push(VerificationEvent::new_error(
            "The voting card set ids are not unique in the list of confirmed encrypted votes (second node)",
        ));
    }
    if first_vc_ids != second_vc_ids {
        res.push(VerificationEvent::new_error(
            "The voting card set ids are not the same in the list of confirmed encrypted votes between first and second node",
        ));
    }
    if res.has_errors() {
        return res;
    }

    // Validate the content for each entry
    for vc_id in first_vc_ids.iter() {
        res.append_with_context(
            &compare_confirmed_enc_vote(
                first
                    .confirmed_encrypted_votes
                    .iter()
                    .find(|v| &v.context_ids.verification_card_id == vc_id)
                    .unwrap(),
                second
                    .confirmed_encrypted_votes
                    .iter()
                    .find(|v| &v.context_ids.verification_card_id == vc_id)
                    .unwrap(),
            ),
            format!("For voting card id {}", vc_id),
        );
    }
    res
}

fn compare_confirmed_enc_vote(
    first: &ConfirmedEncryptedVote,
    second: &ConfirmedEncryptedVote,
) -> VerificationResult {
    let mut res = VerificationResult::new();
    compare_value(
        &first.context_ids.election_event_id,
        &second.context_ids.election_event_id,
        "election_event_id",
        &mut res,
    );
    compare_value(
        &first.context_ids.verification_card_set_id,
        &second.context_ids.verification_card_set_id,
        "verification_card_set_id",
        &mut res,
    );
    compare_value(
        &first.context_ids.verification_card_id,
        &second.context_ids.verification_card_id,
        "verification_card_id",
        &mut res,
    );
    compare_value(
        &first.encrypted_vote.gamma,
        &second.encrypted_vote.gamma,
        "encrypted_vote.gamma",
        &mut res,
    );
    compare_value(
        &first.encrypted_vote.gamma,
        &second.encrypted_vote.gamma,
        "encrypted_vote.gamma",
        &mut res,
    );
    compare_slice(
        &first.encrypted_vote.phis,
        &second.encrypted_vote.phis,
        "encrypted_vote.phis",
        &mut res,
    );
    compare_value(
        &first.exponentiated_encrypted_vote.gamma,
        &second.exponentiated_encrypted_vote.gamma,
        "exponentiated_encrypted_vote.gamma",
        &mut res,
    );
    compare_slice(
        &first.exponentiated_encrypted_vote.phis,
        &second.exponentiated_encrypted_vote.phis,
        "exponentiated_encrypted_vote.phis",
        &mut res,
    );
    compare_value(
        &first.encrypted_partial_choice_return_codes.gamma,
        &second.encrypted_partial_choice_return_codes.gamma,
        "encrypted_partial_choice_return_codes.gamma",
        &mut res,
    );
    compare_slice(
        &first.encrypted_partial_choice_return_codes.phis,
        &second.encrypted_partial_choice_return_codes.phis,
        "encrypted_partial_choice_return_codes.phis",
        &mut res,
    );
    compare_value(
        &first.exponentiation_proof.e,
        &second.exponentiation_proof.e,
        "exponentiation_proof.e",
        &mut res,
    );
    compare_value(
        &first.exponentiation_proof.z,
        &second.exponentiation_proof.z,
        "exponentiation_proof.z",
        &mut res,
    );
    compare_value(
        &first.plaintext_equality_proof.e,
        &second.plaintext_equality_proof.e,
        "plaintext_equality_proof.e",
        &mut res,
    );
    compare_slice(
        &first.plaintext_equality_proof.z,
        &second.plaintext_equality_proof.z,
        "plaintext_equality_proof.z",
        &mut res,
    );
    res
}

fn compare_value<T: PartialEq + Display>(
    value_1: &T,
    value_2: &T,
    name: &str,
    result: &mut VerificationResult,
) {
    if value_1 != value_2 {
        result.push(VerificationEvent::new_failure(&format!(
            "{} not the same. First node: {} / Second node: {}",
            name, value_1, value_2
        )))
    }
}

fn compare_slice<T: PartialEq + Display>(
    slice_1: &[T],
    slice_2: &[T],
    name: &str,
    result: &mut VerificationResult,
) {
    if slice_1.len() != slice_2.len() {
        result.push(VerificationEvent::new_failure(&format!(
            "{} not the same. len not the same",
            name,
        )));
        return;
    }
    for (i, (value_1, value_2)) in slice_1.iter().zip(slice_2.iter()).enumerate() {
        compare_value(
            value_1,
            value_2,
            format!("{}[{}]", name, i).as_str(),
            result,
        );
    }
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
        assert!(result.is_ok());
    }
}
