use rust_ev_crypto_primitives::{
    elgamal::{Ciphertext, EncryptionParameters},
    mix_net::ShuffleArgument,
    Integer,
};
use rust_ev_system_library::{
    preliminaries::{PTable, PTableTrait},
    tally_phase::{
        mix_offline::{
            VerifyMixDecOfflineContext, VerifyMixDecOfflineInput, VerifyMixDecOfflineOutput,
            VerifyVotingClientProofsContext, VerifyVotingClientProofsInput,
            VerifyVotingClientProofsOutput,
        },
        mix_online::{
            GetMixnetInitialCiphertextsContext, GetMixnetInitialCiphertextsInput,
            GetMixnetInitialCiphertextsOuput,
        },
    },
};

use crate::{
    data_structures::common_types::{DecryptionProof, SchnorrProof},
    verification::{VerificationEvent, VerificationResult},
};

/// Context data for algorithm 4.1 according to the specifications
pub struct ContextAlgorithm41<'a> {
    pub eg: &'a EncryptionParameters,
    pub ee_id: &'a str,
    pub vcs_id: &'a str,
    pub bb_id: &'a str,
    pub _upper_n_upper_e: usize,
    pub p_table: &'a PTable,
    pub el_pk: &'a [Integer],
    pub ccm_pk: &'a [&'a [Integer]],
    pub eb_pk: &'a [Integer],
    pub pk_ccr: &'a [Integer],
}

/// Input data for algorithm 4.1 according to the specifications
pub struct InputsAlgorithm41<'a> {
    pub vc_1: &'a [&'a str],
    pub upper_e1_1: &'a [&'a Ciphertext],
    pub upper_e1_1_tilde: &'a [(&'a Integer, &'a Integer)],
    pub upper_e2_1: &'a [&'a Ciphertext],
    pub pi_exp_1: &'a [&'a SchnorrProof],
    pub pi_eq_enc_1: &'a [&'a DecryptionProof],
    pub cs_mix: &'a [&'a [Ciphertext]],
    pub pi_mix: &'a [ShuffleArgument<'a>],
    pub cs_dec: &'a [&'a [Ciphertext]],
    pub pi_dec: &'a [&'a [DecryptionProof]],
    pub vcs: &'a [&'a str],
    pub upper_k: &'a [&'a Integer],
}

pub fn verify_online_control_components_ballot_box(
    context: &ContextAlgorithm41,
    input: &InputsAlgorithm41,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    if !input.vc_1.is_empty() {
        let k_map = input
            .vc_1
            .iter()
            .cloned()
            .zip(input.upper_k.iter().cloned())
            .collect::<Vec<_>>();
        let el_pk = context.el_pk.iter().map(|pk| pk).collect::<Vec<_>>();
        let pk_ccr = context.pk_ccr.iter().map(|pk| pk).collect::<Vec<_>>();
        let pi_exp_1 = input
            .pi_exp_1
            .iter()
            .map(|p| p.as_tuple())
            .collect::<Vec<_>>();
        let pi_eq_enc_1 = input
            .pi_eq_enc_1
            .iter()
            .map(|p| (&p.e, (&p.z[0], &p.z[1])))
            .collect::<Vec<_>>();
        let vc_proofs_verif = VerifyVotingClientProofsOutput::verify_voting_client_proofs(
            &VerifyVotingClientProofsContext {
                encryption_parameters: context.eg,
                ee: context.ee_id,
                vcs: context.vcs_id,
                p_table: context.p_table,
                upper_n_upper_e: context._upper_n_upper_e,
                el_pk: el_pk.as_slice(),
                pk_ccr: pk_ccr.as_slice(),
            },
            &VerifyVotingClientProofsInput {
                vc_1: input.vc_1,
                e1_1: input.upper_e1_1,
                e1_tilde_1: input.upper_e1_1_tilde,
                e2_1: input.upper_e2_1,
                pi_exp_1: pi_exp_1.as_slice(),
                pi_eq_enc_1: pi_eq_enc_1.as_slice(),
                k_map: &k_map,
            },
        );
        result.append_errors_from_string(
            &vc_proofs_verif
                .errors()
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>(),
        );
        result.append_failures_from_string(&vc_proofs_verif.failures());
    }
    let vc_map_1 = input
        .vc_1
        .iter()
        .cloned()
        .zip(input.upper_e1_1.iter().cloned())
        .collect::<Vec<_>>();
    let c_init_1 = match GetMixnetInitialCiphertextsOuput::get_mixnet_initial_ciphertexts(
        &GetMixnetInitialCiphertextsContext {
            eg: context.eg,
            _upper_n_upper_e: context._upper_n_upper_e,
            delta: context.p_table.get_delta(),
            el_pk: context.el_pk,
        },
        &GetMixnetInitialCiphertextsInput {
            vc_map_j: &vc_map_1,
        },
    ) {
        Ok(res) => res.c_init_j,
        Err(e) => {
            result.push(VerificationEvent::new_error(&format!(
                "Error getting initial ciphertext c_init_1: {}",
                e
            )));
            return result;
        }
    };
    let pi_dec = input
        .pi_dec
        .iter()
        .map(|&pis| pis.iter().map(|pi| pi.as_tuple()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let shuffle_proofs_verif = VerifyMixDecOfflineOutput::verify_voting_client_proofs(
        &VerifyMixDecOfflineContext {
            encryption_parameters: context.eg,
            ee: context.ee_id,
            bb: context.bb_id,
            delta: context.p_table.get_delta(),
            el_pk: context.el_pk,
            ccm_pk: context.ccm_pk,
            eb_pk: context.eb_pk,
        },
        &VerifyMixDecOfflineInput {
            c_init_1: &c_init_1,
            c_mix: input.cs_mix,
            pi_mix: input.pi_mix,
            c_dec: input.cs_dec,
            pi_dec: pi_dec.as_slice(),
        },
    );
    result.append_errors_from_string(
        &shuffle_proofs_verif
            .errors()
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>(),
    );
    result.append_failures_from_string(&shuffle_proofs_verif.failures());
    result
}
