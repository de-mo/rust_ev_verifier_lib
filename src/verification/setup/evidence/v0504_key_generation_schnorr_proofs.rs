use super::super::super::result::{
    create_verification_error, create_verification_failure, VerificationEvent, VerificationResult,
};
use crate::{
    config::Config,
    data_structures::common_types::Proof,
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};
use anyhow::anyhow;
use log::debug;
use rayon::prelude::*;
use rug::Integer;
use rust_ev_crypto_primitives::{verify_schnorr, EncryptionParameters};
use std::iter::zip;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let ee_context = match context_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(create_verification_error!(
                "election_event_context_payload cannot be read",
                e
            ));
            return;
        }
    };
    let setup_ppk = match context_dir.setup_component_public_keys_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(create_verification_error!(
                "setup_component_public_keys_payload cannot be read",
                e
            ));
            return;
        }
    };

    // CC proofs
    for combined_cc_pk in setup_ppk
        .setup_component_public_keys
        .combined_control_component_public_keys
    {
        let j = combined_cc_pk.node_id;

        // CCRj Schnorr Proofs
        let i_aux_ccr_j = vec![
            ee_context.election_event_context.election_event_id.clone(),
            "GenKeysCCR".to_string(),
            j.to_string(),
        ];
        let proofs: Vec<Proof> = combined_cc_pk
            .ccrj_schnorr_proofs
            .iter()
            .map(Proof::from)
            .collect();

        let mut res = run_verify_schnorr_proofs(
            &ee_context.encryption_group,
            &combined_cc_pk.ccrj_choice_return_codes_encryption_public_key,
            &proofs,
            &i_aux_ccr_j,
        )
        .add_context("Test VerifSchnorrCCRji")
        .add_context("Proof CCR_j")
        .add_context(format!("node {}", j));
        result.append(&mut res);

        // CCMj Schnorr Proofs
        let i_aux_ccm_j = vec![
            ee_context.election_event_context.election_event_id.clone(),
            "SetupTallyCCM".to_string(),
            j.to_string(),
        ];
        let proofs: Vec<Proof> = combined_cc_pk
            .ccmj_schnorr_proofs
            .iter()
            .map(Proof::from)
            .collect();

        let mut res = run_verify_schnorr_proofs(
            &ee_context.encryption_group,
            &combined_cc_pk.ccmj_election_public_key,
            &proofs,
            &i_aux_ccm_j,
        )
        .add_context("Test VerifSchnorrCCMji")
        .add_context("Proof CCM_j")
        .add_context(format!("node {}", j));
        result.append(&mut res);
    }

    // EB proofs
    let i_aux_eb = vec![
        ee_context.election_event_context.election_event_id.clone(),
        "SetupTallyEB".to_string(),
    ];
    let proofs: Vec<Proof> = setup_ppk
        .setup_component_public_keys
        .electoral_board_schnorr_proofs
        .iter()
        .map(Proof::from)
        .collect();
    let mut res = run_verify_schnorr_proofs(
        &ee_context.encryption_group,
        &setup_ppk
            .setup_component_public_keys
            .electoral_board_public_key,
        &proofs,
        &i_aux_eb,
    )
    .add_context("Test VerifSchnorrELi")
    .add_context("Proof Electoral board");
    result.append(&mut res);
}

fn run_verify_schnorr_proofs(
    eg: &EncryptionParameters,
    pks: &Vec<Integer>,
    pis: &Vec<Proof>,
    i_aux: &Vec<String>,
) -> VerificationResult {
    let mut res = VerificationResult::new();
    if pks.len() != pis.len() {
        res.push(create_verification_error!(
            "The length of pks and pis is not the same"
        ));
    } else {
        let failures: Vec<Option<VerificationEvent>> = zip(pks, pis)
            .enumerate()
            .par_bridge()
            .map(|(i, (pk, pi))| {
                run_verify_schnorr_proof(eg, pi, pk, i_aux)
                    .map(|e| e.add_context(format!("at position {}", i)))
            })
            .collect();
        failures.into_iter().for_each(|o| {
            if let Some(f) = o {
                res.push(f)
            }
        });
    }
    res
}

#[allow(clippy::too_many_arguments)]
fn run_verify_schnorr_proof(
    eg: &EncryptionParameters,
    schnorr: &Proof,
    y: &Integer,
    i_aux: &Vec<String>,
) -> Option<VerificationEvent> {
    match verify_schnorr(eg, schnorr.as_tuple(), y, i_aux) {
        Err(e) => return Some(VerificationEvent::failure_from_error(e)),
        Ok(b) => {
            if !b {
                return Some(create_verification_failure!("Schnorr proofs not ok"));
            }
        }
    }
    None
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
