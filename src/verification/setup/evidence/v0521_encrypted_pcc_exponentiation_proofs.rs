use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailure, VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    crypto_primitives::zero_knowledge_proof::verify_exponentiation,
    data_structures::{
        common_types::EncryptionGroup,
        setup::{
            control_component_code_shares_payload::ControlComponentCodeShare,
            setup_component_verification_data_payload::SetupComponentVerificationDataInner,
        },
        VerifierSetupDataTrait,
    },
    error::{create_verifier_error, VerifierError},
    file_structure::{
        setup_directory::{SetupDirectoryTrait, VCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::verification::VerificationResultTrait,
};
use log::debug;
use rayon::prelude::*;
use std::iter::zip;

/// Context data according to the specifications
struct Context<'a> {
    eg: &'a EncryptionGroup,
    node_id: &'a usize,
    ee_id: &'a String,
    vcs_id: &'a String,
    //nb_voters: &'a usize,
    nb_voting_options: &'a usize,
    chunk_id: &'a usize,
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();

    // Read ee context for the context of the algorithm
    let ee_context = match setup_dir.election_event_context_payload() {
        Ok(p) => p.election_event_context,
        Err(e) => {
            result.push_error(create_verification_error!(
                "election_event_context_payload cannot be read",
                e
            ));
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
                vcs_dir.get_name(),
                chunk_id
            );
            let cc_share_chunk_name = format!(
                "{}/control_component_code_shares_payload.{}",
                vcs_dir.get_name(),
                chunk_id
            );
            match setup_verification_data_payload_result {
                Ok(setup_verification_data_payload) => {
                    let vcs_id = &setup_verification_data_payload.verification_card_set_id;
                    // Find correct vcs context
                    let vcs_context = match ee_context.find_verification_card_set_context(&vcs_id) {
                        Some(c) => c,
                        None => {
                            result.push_error(create_verification_error!(format!(
                                "vcs id {} not found in election_event_context_payload",
                                vcs_id
                            )));
                            break;
                        }
                    };
                    let verification_card_ids =
                        setup_verification_data_payload.verification_card_ids();
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
                            result.push_failure(create_verification_failure!(
                                format!("Number of vcs {} for vcs id {} not the same as in the chunks of setup_verification_data",
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
                        .get_data()
                    {
                        Ok(s) => {
                            let cc_shares = s.control_component_code_shares_payload().unwrap();
                            // For each CC (1 to 4)
                            let mut res_cc: Vec<VerificationResult> = (1usize..=4usize)
                                .par_bridge()
                                .map(|j| {
                                    let mut result = VerificationResult::new();
                                    // find the correct node
                                    match cc_shares.iter().find(|s| s.node_id == j) {
                                        Some(s) => {
                                            let context = Context {
                                                eg: &setup_verification_data_payload
                                                    .encryption_group,
                                                node_id: &j,
                                                ee_id: &setup_verification_data_payload
                                                    .election_event_id,
                                                vcs_id,
                                                //nb_voters: &vcs_context.number_of_voters(),
                                                nb_voting_options: &vcs_context
                                                    .number_of_voting_options(),
                                                chunk_id: &chunk_id,
                                            };
                                            result.append(
                                                &mut verify_encrypted_pccexponentiation_proofs(
                                                    &context,
                                                    &verification_card_ids,
                                                    &setup_verification_data_payload
                                                        .setup_component_verification_data,
                                                    &s.control_component_code_shares,
                                                ),
                                            )
                                        }
                                        None => {
                                            result.push_error(create_verification_error!(format!(
                                                "Node id {} not found in {}",
                                                j, cc_share_chunk_name
                                            )))
                                        }
                                    }
                                    result
                                })
                                .collect();
                            for r in res_cc.iter_mut() {
                                result.append(r);
                            }
                        }
                        Err(e) => result.push_error(create_verification_error!(
                            format!("{} cannot be read", cc_share_chunk_name),
                            e
                        )),
                    }
                }
                Err(e) => {
                    result.push_error(create_verification_error!(
                        format!("{} cannot be read", setup_verif_data_chunk_name),
                        e
                    ));
                }
            }
        }
    }
}

/// Supporting algorithm
fn verify_encrypted_pccexponentiation_proofs(
    context: &Context,
    verification_card_ids: &Vec<&String>,
    setup_verif_data: &Vec<SetupComponentVerificationDataInner>,
    cc_code_shares: &Vec<ControlComponentCodeShare>,
) -> VerificationResult {
    let mut result: VerificationResult = VerificationResult::new();
    debug!(
        "Verification for vcs_id {}, chunk {} and node {}",
        context.vcs_id, context.chunk_id, context.node_id
    );
    if verify_sizes(
        &verification_card_ids.len(),
        vec![setup_verif_data.len(), cc_code_shares.len()],
        result.failures_mut(),
        vec!["setup_verif_data", "cc_code_shares"],
        &format!("for chunk {}", context.chunk_id),
    ) {
        // Parallel verification for each voting card.
        // WARNING: It is assumed that the voting cards are in the same order in each list.
        let mut failures: Vec<Vec<VerificationFailure>> = verification_card_ids
            .iter()
            .enumerate()
            .par_bridge()
            .map(|(i, vc_id)| {
                verify_encrypted_pccexponentiation_proofs_for_one_vc(
                    context,
                    vc_id,
                    &setup_verif_data[i],
                    &cc_code_shares[i],
                )
            })
            .collect();
        for fs in failures.iter_mut() {
            result.failures_mut().append(fs);
        }
    }
    result
}

/// Supporting algorithm for one vc
fn verify_encrypted_pccexponentiation_proofs_for_one_vc(
    context: &Context,
    vc_id: &String,
    setup_verif: &SetupComponentVerificationDataInner,
    cc_code_share: &ControlComponentCodeShare,
) -> Vec<VerificationFailure> {
    let mut failures = vec![];

    debug!(
        "Verification for vcs_id {}, chunk {}, node {} and vc_id {}",
        context.vcs_id, context.chunk_id, context.node_id, vc_id
    ); // Verify that the size of phis correspond to the voting options
    if verify_sizes(
        context.nb_voting_options,
        vec![
            setup_verif
                .encrypted_hashed_squared_partial_choice_return_codes
                .phis
                .len(),
            cc_code_share
                .exponentiated_encrypted_partial_choice_return_codes
                .phis
                .len(),
        ],
        &mut failures,
        vec![
            "setup_verif.encrypted_hashed_squared_partial_choice_return_codes.phis",
            "cc_code_share.exponentiated_encrypted_partial_choice_return_codes.phis",
        ],
        &format!("for chunk {} and voting card {}", context.chunk_id, vc_id),
    ) {
        let mut gs = setup_verif
            .encrypted_hashed_squared_partial_choice_return_codes
            .phis
            .clone();
        gs.insert(
            0,
            setup_verif
                .encrypted_hashed_squared_partial_choice_return_codes
                .gamma
                .clone(),
        );
        gs.insert(0, context.eg.g.clone());
        let mut ys = cc_code_share
            .exponentiated_encrypted_partial_choice_return_codes
            .phis
            .clone();
        ys.insert(
            0,
            cc_code_share
                .exponentiated_encrypted_partial_choice_return_codes
                .gamma
                .clone(),
        );
        ys.insert(
            0,
            cc_code_share.voter_choice_return_code_generation_public_key[0].clone(),
        );
        let i_aux = vec![
            context.ee_id.clone(),
            vc_id.clone(),
            "GenEncLongCodeShares".to_string(),
            context.node_id.to_string(),
        ];
        let pi_exp_pcc_j = cc_code_share
            .encrypted_partial_choice_return_code_exponentiation_proof
            .clone();
        if !verify_exponentiation(context.eg, &gs, &ys, &pi_exp_pcc_j, &i_aux) {
            failures.push(create_verification_failure!(format!(
                "Failure verifying proofs for voting card id {} in chunk {} for node {}",
                vc_id, context.chunk_id, context.node_id
            )))
        };
    }
    failures
}

/// Verify the size of many arrays
///
/// Names must have the same length of vec_sizes
fn verify_sizes(
    expected: &usize,
    vec_sizes: Vec<usize>,
    result: &mut Vec<VerificationFailure>,
    names: Vec<&str>,
    suffix_text: &String,
) -> bool {
    let mut res = true;
    for (size, name) in zip(vec_sizes, names) {
        if &size != expected {
            result.push(create_verification_failure!(format!(
                "number of {} {} not equal to number of voters {} {}",
                name, size, expected, suffix_text
            )));
            res = false;
        }
    }
    res
}

#[cfg(test)]
mod test {
    use super::{
        super::super::super::{verification::VerificationResultTrait, VerificationPeriod},
        *,
    };
    use crate::file_structure::VerificationDirectory;
    use std::path::Path;

    fn get_verifier_dir() -> VerificationDirectory {
        let location = Path::new(".").join("datasets").join("dataset1-setup-tally");
        VerificationDirectory::new(&VerificationPeriod::Setup, &location)
    }

    #[test]
    #[ignore]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &mut result);
        assert!(result.is_ok().unwrap());
    }
}
