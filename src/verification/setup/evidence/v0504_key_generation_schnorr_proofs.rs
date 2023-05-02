use super::super::super::{
    error::{
        create_verification_error, create_verification_failure, VerificationErrorType,
        VerificationFailure, VerificationFailureType,
    },
    verification::VerificationResult,
};
use crate::{
    crypto_primitives::zero_knowledge_proof::verify_schnorr,
    data_structures::common_types::{EncryptionGroup, Proof},
    error::{create_verifier_error, VerifierError},
    file_structure::{setup_directory::SetupDirectoryTrait, VerificationDirectoryTrait},
};
use num_bigint::BigUint;
use rayon::prelude::*;
use std::iter::zip;

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    result: &mut VerificationResult,
) {
    let setup_dir = dir.unwrap_setup();
    let eg = match setup_dir.encryption_parameters_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push_error(create_verification_error!(
                "encryption_parameters_payload cannot be read",
                e
            ));
            return;
        }
    };
    let ee_context = match setup_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push_error(create_verification_error!(
                "election_event_context_payload cannot be read",
                e
            ));
            return;
        }
    };
    let setup_ppk = match setup_dir.setup_component_public_keys_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push_error(create_verification_error!(
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
            .map(|e| Proof::from(e))
            .collect();

        let mut res = run_verify_schnorr_proofs(
            &eg.encryption_group,
            &combined_cc_pk.ccrj_choice_return_codes_encryption_public_key,
            &proofs,
            &i_aux_ccr_j,
            "VerifSchnorrCCRji",
            "CCR_j",
            &Some(j),
        );
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
            .map(|e| Proof::from(e))
            .collect();

        let mut res = run_verify_schnorr_proofs(
            &eg.encryption_group,
            &combined_cc_pk.ccmj_election_public_key,
            &proofs,
            &i_aux_ccm_j,
            "VerifSchnorrCCMji",
            "CCM_j",
            &Some(j),
        );
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
        .map(|e| Proof::from(e))
        .collect();
    let mut res = run_verify_schnorr_proofs(
        &eg.encryption_group,
        &setup_ppk
            .setup_component_public_keys
            .electoral_board_public_key,
        &proofs,
        &i_aux_eb,
        "VerifSchnorrELi",
        "Electoral board",
        &None,
    );
    result.append(&mut res);
}

fn run_verify_schnorr_proofs(
    eg: &EncryptionGroup,
    pks: &Vec<BigUint>,
    pis: &Vec<Proof>,
    i_aux: &Vec<String>,
    test_name: &str,
    proof_name: &str,
    node: &Option<usize>,
) -> VerificationResult {
    let mut res = VerificationResult::new();
    if pks.len() != pis.len() {
        res.push_error(create_verification_error!(format!(
            "The length of pks and pis is not the same for {proof_name}"
        )));
    } else {
        let failures: Vec<Option<VerificationFailure>> = zip(pks, pis)
            .enumerate()
            .par_bridge()
            .map(|(i, (pk, pi))| {
                run_verify_schnorr_proof(eg, pi, &pk, &i_aux, test_name, proof_name, i, node)
            })
            .collect();
        for o in failures {
            match o {
                Some(f) => res.push_failure(f),
                None => (),
            }
        }
    }
    res
}

fn run_verify_schnorr_proof(
    eg: &EncryptionGroup,
    schnorr: &Proof,
    y: &BigUint,
    i_aux: &Vec<String>,
    test_name: &str,
    proof_name: &str,
    pos: usize,
    node: &Option<usize>,
) -> Option<VerificationFailure> {
    println!("Run {} at pos {} for cc {:?}", test_name, pos, node);
    if !verify_schnorr(eg, schnorr, y, i_aux) {
        let mut text = format!(
            "{}: Verifiy {} Schnorr proofs not ok at pos {}",
            test_name, proof_name, pos
        );
        if node.is_some() {
            text = format!("{} for node {}", text, node.unwrap());
        }
        return Some(create_verification_failure!(text));
    }
    None
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
