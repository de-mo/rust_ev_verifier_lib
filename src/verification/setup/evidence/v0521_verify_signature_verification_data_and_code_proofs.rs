use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    data_structures::{
        common_types::SchnorrProof,
        setup::{
            control_component_code_shares_payload::ControlComponentCodeSharesPayloadInner,
            setup_component_verification_data_payload::{
                SetupComponentVerificationDataInner, SetupComponentVerificationDataPayload,
            },
        },
        VerifierSetupDataTrait,
    },
    file_structure::{
        context_directory::ContextDirectoryTrait,
        setup_directory::{SetupDirectoryTrait, SetupVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::verify_signature_for_object,
};

use rayon::prelude::*;
use rust_ev_crypto_primitives::{elgamal::Ciphertext, Integer};
use rust_ev_crypto_primitives::{
    elgamal::EncryptionParameters, zero_knowledge_proofs::verify_exponentiation,
};
use std::iter::zip;

/// Context data for algorithm 3.3 according to the specifications
struct ContextAlgorithm33<'a, 'b, 'c, 'd, 'e> {
    eg: &'a EncryptionParameters,
    node_id: usize,
    ee_id: &'b String,
    vc_ids: &'c Vec<&'d String>,
    _setup_component_verification_data: &'e Vec<SetupComponentVerificationDataInner>,
    nb_voting_options: usize,
}

/// Context data for algorithm 3.4 according to the specifications
struct ContextAlgorithm34<'a, 'b, 'c, 'd, 'e> {
    eg: &'a EncryptionParameters,
    node_id: usize,
    ee_id: &'b String,
    vc_ids: &'c Vec<&'d String>,
    _setup_component_verification_data: &'e Vec<SetupComponentVerificationDataInner>,
}

fn algorithm_0301_verify_signature_setup_component_verification_data(
    verification_data_payload: &SetupComponentVerificationDataPayload,
    config: &'static Config,
) -> VerificationResult {
    verify_signature_for_object(verification_data_payload, config)
}

fn algorithm_0302_verify_signature_control_component_code_shares(
    control_component_code_shares: &ControlComponentCodeSharesPayloadInner,
    config: &'static Config,
) -> VerificationResult {
    verify_signature_for_object(control_component_code_shares, config)
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let setup_dir = dir.unwrap_setup();

    // Read ee context for the context of the algorithm
    let ee_context = match context_dir.election_event_context_payload() {
        Ok(p) => p.election_event_context,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };

    // For each vcs directory
    for vcs_dir in setup_dir.vcs_directories() {
        // Values over chuncks to check the number of voting cards
        let mut vcs_id_for_sum_vcs = String::new();
        let mut current_sum_vcs = 0usize;
        // For each chunk
        for (chunk_id, setup_verification_data_payload_result) in
            vcs_dir.setup_component_verification_data_payload_iter()
        {
            let setup_verif_data_chunk_name = format!(
                "{}/setup_component_verification_data_payload.{}",
                vcs_dir.name(),
                chunk_id
            );
            let cc_share_chunk_name = format!(
                "{}/control_component_code_shares_payload.{}",
                vcs_dir.name(),
                chunk_id
            );
            if let Err(e) = setup_verification_data_payload_result {
                result.push(
                    VerificationEvent::new_error(&e)
                        .add_context(format!("{} cannot be read", setup_verif_data_chunk_name)),
                );
                break;
            }
            let setup_verification_data_payload = setup_verification_data_payload_result.unwrap();

            // Verify signature setup_component_verification_data
            result.append_with_context(
                &algorithm_0301_verify_signature_setup_component_verification_data(
                    &setup_verification_data_payload,
                    config,
                ),
                setup_verif_data_chunk_name.clone(),
            );

            let vcs_id = &setup_verification_data_payload.verification_card_set_id;
            // Find correct vcs context
            let vcs_context = match ee_context.find_verification_card_set_context(vcs_id) {
                Some(c) => c,
                None => {
                    result.push(VerificationEvent::new_error(&format!(
                        "vcs id {} not found in election_event_context_payload",
                        vcs_id
                    )));
                    break;
                }
            };
            let verification_card_ids = setup_verification_data_payload.verification_card_ids();
            // Check the number of voters of the vcs_id changed in the new chunk
            if &vcs_id_for_sum_vcs != vcs_id {
                // vcs_id changed
                if !vcs_id_for_sum_vcs.is_empty()
                    && current_sum_vcs
                        != ee_context
                            .find_verification_card_set_context(&vcs_id_for_sum_vcs)
                            .unwrap()
                            .number_of_voters()
                {
                    result.push(VerificationEvent::new_failure(
                                &format!("Number of vcs {} for vcs id {} not the same as in the chunks of setup_verification_data",
                                vcs_context.number_of_voters(), vcs_id_for_sum_vcs)
                            ));
                };
                vcs_id_for_sum_vcs = vcs_id.clone();
                current_sum_vcs = verification_card_ids.len();
            } else {
                // vcs_id not changed
                current_sum_vcs += verification_card_ids.len();
            }
            // Get cc_shares from the same chunk
            match vcs_dir
                .control_component_code_shares_payload_group()
                .get_file_with_number(chunk_id)
                .get_verifier_data()
            {
                Ok(s) => {
                    let cc_shares = s.control_component_code_shares_payload().unwrap();
                    // For each CC (1 to 4)
                    let mut res_cc: Vec<VerificationResult> = cc_shares.0
                                .iter()
                                .par_bridge()
                                .map(|shares| {
                                    let mut result = VerificationResult::new();
                                    result.append_with_context(&algorithm_0302_verify_signature_control_component_code_shares(
                                        shares,
                                        config),
                                        format!("{}.{}", cc_share_chunk_name, shares.node_id),
                                    );
                                    let context33 = ContextAlgorithm33 {
                                        eg: &setup_verification_data_payload.encryption_group,
                                        node_id: shares.node_id,
                                        ee_id: &setup_verification_data_payload.election_event_id,
                                        vc_ids: &verification_card_ids,
                                        nb_voting_options: vcs_context.number_of_voting_options(),
                                        _setup_component_verification_data: &setup_verification_data_payload.setup_component_verification_data
                                    };
                                    let mut res_algo_33 = algorithm_0303_verify_encrypted_pcc_exponentiation_proofs_verification_card_set(
                                        &context33,
                                        setup_verification_data_payload.setup_component_verification_data.iter().map(|e| &e.encrypted_hashed_squared_partial_choice_return_codes).collect(),
                                        shares.control_component_code_shares.iter().map(|e| &e.voter_choice_return_code_generation_public_key[0]).collect(),
                                        shares.control_component_code_shares.iter().map(|e| &e.exponentiated_encrypted_partial_choice_return_codes).collect(),
                                        shares.control_component_code_shares.iter().map(|e| &e.encrypted_partial_choice_return_code_exponentiation_proof).collect(),
                                    ).clone_add_context(format!("Node {}", shares.node_id)).clone_add_context(format!("Chunk {}", cc_share_chunk_name));
                                    result.append(&mut res_algo_33);
                                    let context34 = ContextAlgorithm34 {
                                        eg: &setup_verification_data_payload.encryption_group,
                                        node_id: shares.node_id,
                                        ee_id: &setup_verification_data_payload.election_event_id,
                                        vc_ids: &verification_card_ids,
                                        _setup_component_verification_data: &setup_verification_data_payload.setup_component_verification_data
                                    };
                                    let mut res_algo_34 = algorithm_0304_verify_encrypted_ckexponentiation_proofs_verification_card_set(
                                        &context34,
                                        setup_verification_data_payload.setup_component_verification_data.iter().map(|e| &e.encrypted_hashed_squared_confirmation_key).collect(),
                                        shares.control_component_code_shares.iter().map(|e| &e.voter_vote_cast_return_code_generation_public_key[0]).collect(),
                                        shares.control_component_code_shares.iter().map(|e| &e.exponentiated_encrypted_confirmation_key).collect(),
                                        shares.control_component_code_shares.iter().map(|e| &e.encrypted_confirmation_key_exponentiation_proof).collect(),
                                    ).clone_add_context(format!("Node {}", shares.node_id)).clone_add_context(format!("Chunk {}", cc_share_chunk_name));
                                    result.append(&mut res_algo_34);
                                    result
                                })
                                .collect();
                    for r in res_cc.iter_mut() {
                        result.append(r);
                    }
                }
                Err(e) => result.push(
                    VerificationEvent::new_error(&e)
                        .add_context(format!("{} cannot be read", cc_share_chunk_name)),
                ),
            }
        }
    }
}

/// Supporting algorithm
fn algorithm_0303_verify_encrypted_pcc_exponentiation_proofs_verification_card_set(
    context: &ContextAlgorithm33,
    encrypted_hashed_squared_partial_choice_return_codes: Vec<&Ciphertext>,
    voter_choice_return_code_generation_public_key: Vec<&Integer>,
    exponentiated_encrypted_partial_choice_return_codes: Vec<&Ciphertext>,
    encrypted_partial_choice_return_code_exponentiation_proof: Vec<&SchnorrProof>,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    let verif_size_res = verify_sizes(
        &context.vc_ids.len(),
        vec![
            encrypted_hashed_squared_partial_choice_return_codes.len(),
            voter_choice_return_code_generation_public_key.len(),
            exponentiated_encrypted_partial_choice_return_codes.len(),
            encrypted_partial_choice_return_code_exponentiation_proof.len(),
        ],
        vec![
            "encrypted_hashed_squared_partial_choice_return_codes".to_string(),
            "voter_choice_return_code_generation_public_key".to_string(),
            "exponentiated_encrypted_partial_choice_return_codes".to_string(),
            "encrypted_partial_choice_return_code_exponentiation_proof".to_string(),
        ],
    );
    if !verif_size_res.is_ok() {
        return verif_size_res;
    }
    // Parallel verification for each voting card.
    // WARNING: It is assumed that the voting cards are in the same order in each list.
    let mut failures: Vec<VerificationResult> = context.vc_ids
            .iter()
            .enumerate()
            .par_bridge()
            .map(|(i, vc_id)| {
                algorithm_0303_verify_encrypted_pcc_exponentiation_proofs_verification_card_set_for_one_vc(
                    context,
                    vc_id,
                    encrypted_hashed_squared_partial_choice_return_codes[i],
                    voter_choice_return_code_generation_public_key[i],
                    exponentiated_encrypted_partial_choice_return_codes[i],
                    encrypted_partial_choice_return_code_exponentiation_proof[i]
                ).clone_add_context(format!("For vc_id {}", vc_id)).clone_add_context(format!("At position {}", i))
            })
            .collect();
    for fs in failures.iter_mut() {
        result.append(fs);
    }
    result
}

/// Supporting algorithm for one vc
fn algorithm_0303_verify_encrypted_pcc_exponentiation_proofs_verification_card_set_for_one_vc(
    context: &ContextAlgorithm33,
    vc_id: &str,
    encrypted_hashed_squared_partial_choice_return_codes: &Ciphertext,
    voter_choice_return_code_generation_public_key: &Integer,
    exponentiated_encrypted_partial_choice_return_codes: &Ciphertext,
    encrypted_partial_choice_return_code_exponentiation_proof: &SchnorrProof,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    let verif_size_res = verify_sizes(
        &context.nb_voting_options,
        vec![
            encrypted_hashed_squared_partial_choice_return_codes
                .phis
                .len(),
            exponentiated_encrypted_partial_choice_return_codes
                .phis
                .len(),
        ],
        vec![
            "setup_verif.encrypted_hashed_squared_partial_choice_return_codes.phis".to_string(),
            "cc_code_share.exponentiated_encrypted_partial_choice_return_codes.phis".to_string(),
        ],
    );
    if !verif_size_res.is_ok() {
        return verif_size_res;
    }
    {
        let mut gs = encrypted_hashed_squared_partial_choice_return_codes
            .phis
            .clone();
        gs.insert(
            0,
            encrypted_hashed_squared_partial_choice_return_codes
                .gamma
                .clone(),
        );
        gs.insert(0, context.eg.g().clone());
        let mut ys = exponentiated_encrypted_partial_choice_return_codes
            .phis
            .clone();
        ys.insert(
            0,
            exponentiated_encrypted_partial_choice_return_codes
                .gamma
                .clone(),
        );
        ys.insert(0, voter_choice_return_code_generation_public_key.clone());
        let i_aux = vec![
            context.ee_id.clone(),
            vc_id.to_string(),
            "GenEncLongCodeShares".to_string(),
            context.node_id.to_string(),
        ];
        let pi_exp_pcc_j = encrypted_partial_choice_return_code_exponentiation_proof.clone();
        match verify_exponentiation(context.eg, &gs, &ys, pi_exp_pcc_j.as_tuple(), &i_aux) {
            Err(e) => result.push(VerificationEvent::new_failure(&e)),
            Ok(b) => {
                if !b {
                    result.push(VerificationEvent::new_failure(
                        "Failure verifying encrypted_pcc_exponentiation_proofs",
                    ))
                }
            }
        }
    }
    result
}

/// Supporting algorithm
fn algorithm_0304_verify_encrypted_ckexponentiation_proofs_verification_card_set(
    context: &ContextAlgorithm34,
    encrypted_hashed_squared_confirmation_key: Vec<&Ciphertext>,
    voter_vote_cast_return_code_generation_public_key: Vec<&Integer>,
    exponentiated_encrypted_confirmation_key: Vec<&Ciphertext>,
    encrypted_confirmation_key_exponentiation_proof: Vec<&SchnorrProof>,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    let verif_size_res = verify_sizes(
        &context.vc_ids.len(),
        vec![
            encrypted_hashed_squared_confirmation_key.len(),
            voter_vote_cast_return_code_generation_public_key.len(),
            exponentiated_encrypted_confirmation_key.len(),
            encrypted_confirmation_key_exponentiation_proof.len(),
        ],
        vec![
            "encrypted_hashed_squared_confirmation_key".to_string(),
            "voter_vote_cast_return_code_generation_public_key".to_string(),
            "exponentiated_encrypted_confirmation_key".to_string(),
            "encrypted_confirmation_key_exponentiation_proof".to_string(),
        ],
    );
    if !verif_size_res.is_ok() {
        return verif_size_res;
    }
    // Parallel verification for each voting card.
    // WARNING: It is assumed that the voting cards are in the same order in each list.
    let mut failures: Vec<VerificationResult> = context.vc_ids
            .iter()
            .enumerate()
            .par_bridge()
            .map(|(i, vc_id)| {
                algorithm_0304_verify_encrypted_ckexponentiation_proofs_verification_card_set_for_one_vc(
                    context,
                    vc_id,
                    encrypted_hashed_squared_confirmation_key[i],
                    voter_vote_cast_return_code_generation_public_key[i],
                    exponentiated_encrypted_confirmation_key[i],
                    encrypted_confirmation_key_exponentiation_proof[i]
                ).clone_add_context(format!("For vc_id {}", vc_id)).clone_add_context(format!("At position {}", i))
            })
            .collect();
    for fs in failures.iter_mut() {
        result.append(fs);
    }
    result
}

/// Supporting algorithm for one vc
fn algorithm_0304_verify_encrypted_ckexponentiation_proofs_verification_card_set_for_one_vc(
    context: &ContextAlgorithm34,
    vc_id: &str,
    encrypted_hashed_squared_confirmation_key: &Ciphertext,
    voter_vote_cast_return_code_generation_public_key: &Integer,
    exponentiated_encrypted_confirmation_key: &Ciphertext,
    encrypted_confirmation_key_exponentiation_proof: &SchnorrProof,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    let verif_size_res = verify_sizes(
        &1usize,
        vec![
            encrypted_hashed_squared_confirmation_key.phis.len(),
            exponentiated_encrypted_confirmation_key.phis.len(),
        ],
        vec![
            "setup_verif.encrypted_hashed_squared_confirmation_key.phis".to_string(),
            "cc_code_share.exponentiated_encrypted_confirmation_key.phis".to_string(),
        ],
    );
    if !verif_size_res.is_ok() {
        return verif_size_res;
    }
    {
        let mut gs = encrypted_hashed_squared_confirmation_key.phis.clone();
        gs.insert(0, encrypted_hashed_squared_confirmation_key.gamma.clone());
        gs.insert(0, context.eg.g().clone());
        let mut ys = exponentiated_encrypted_confirmation_key.phis.clone();
        ys.insert(0, exponentiated_encrypted_confirmation_key.gamma.clone());
        ys.insert(0, voter_vote_cast_return_code_generation_public_key.clone());
        let i_aux = vec![
            context.ee_id.clone(),
            vc_id.to_string(),
            "GenEncLongCodeShares".to_string(),
            context.node_id.to_string(),
        ];
        let pi_exp_pcc_j = encrypted_confirmation_key_exponentiation_proof.clone();
        match verify_exponentiation(context.eg, &gs, &ys, pi_exp_pcc_j.as_tuple(), &i_aux) {
            Err(e) => result.push(VerificationEvent::new_failure(&e)),
            Ok(b) => {
                if !b {
                    result.push(VerificationEvent::new_failure(
                        "Failure verifying encrypted_ck_exponentiation_proofs",
                    ))
                }
            }
        }
    }
    result
}

/// Verify the size of many arrays, the must be the same as expected
///
/// Names must have the same length of vec_sizes
fn verify_sizes(expected: &usize, vec_sizes: Vec<usize>, names: Vec<String>) -> VerificationResult {
    let mut result = VerificationResult::new();
    for (size, name) in zip(vec_sizes, names) {
        if &size != expected {
            result.push(VerificationEvent::new_failure(&format!(
                "number of elements {} in {} are not equal to expected {}",
                size, name, expected
            )));
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
