use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};
use rust_ev_system_library::preliminaries::{
    GetHashElectionEventContextContext, VerifyKeyGenerationSchnorrProofsInput,
    VerifyKeyGenerationSchnorrProofsOuput,
};

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let ee_context = match context_dir.election_event_context_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };
    let setup_cc_ppk_payload = match context_dir.setup_component_public_keys_payload() {
        Ok(eg) => eg,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("setup_component_public_keys_payload cannot be read"),
            );
            return;
        }
    };

    let get_hash_election_event_context =
        GetHashElectionEventContextContext::from(&ee_context.election_event_context);

    // Prepare inputs
    let pk_ccr = setup_cc_ppk_payload
        .setup_component_public_keys
        .combined_control_component_public_keys
        .iter()
        .map(|cc| {
            cc.ccrj_choice_return_codes_encryption_public_key
                .iter()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let pi_pkccr = setup_cc_ppk_payload
        .setup_component_public_keys
        .combined_control_component_public_keys
        .iter()
        .map(|cc| {
            cc.ccrj_schnorr_proofs
                .iter()
                .map(|p| p.as_tuple())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let el_pk = setup_cc_ppk_payload
        .setup_component_public_keys
        .combined_control_component_public_keys
        .iter()
        .map(|cc| cc.ccmj_election_public_key.iter().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let pi_elpk = setup_cc_ppk_payload
        .setup_component_public_keys
        .combined_control_component_public_keys
        .iter()
        .map(|cc| {
            cc.ccmj_schnorr_proofs
                .iter()
                .map(|p| p.as_tuple())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let eb_pk = setup_cc_ppk_payload
        .setup_component_public_keys
        .electoral_board_public_key
        .iter()
        .collect::<Vec<_>>();
    let pi_eb = setup_cc_ppk_payload
        .setup_component_public_keys
        .electoral_board_schnorr_proofs
        .iter()
        .map(|p| p.as_tuple())
        .collect::<Vec<_>>();
    let verify_key_generation_schnorr_proofs_input = VerifyKeyGenerationSchnorrProofsInput {
        pk_ccr: pk_ccr.as_slice(),
        pi_pkccr: pi_pkccr.as_slice(),
        el_pk: el_pk.as_slice(),
        pi_elpk: pi_elpk.as_slice(),
        eb_pk: &eb_pk,
        pi_eb: &pi_eb,
    };

    let verif_schnorr_key_generation =
        VerifyKeyGenerationSchnorrProofsOuput::verify_key_generation_schnorr_proofs(
            &get_hash_election_event_context,
            &verify_key_generation_schnorr_proofs_input,
        );

    result.extend(
        verif_schnorr_key_generation
            .errors
            .iter()
            .map(VerificationEvent::new_error)
            .chain(
                verif_schnorr_key_generation
                    .verif_schnorr_ccm
                    .iter()
                    .map(|e| VerificationEvent::new_error(e).add_context("verif_schnorr_ccm")),
            )
            .chain(
                verif_schnorr_key_generation
                    .verif_schnorr_ccr
                    .iter()
                    .map(|e| VerificationEvent::new_error(e).add_context("verif_schnorr_ccr")),
            )
            .chain(
                verif_schnorr_key_generation
                    .verif_schnorr_eb
                    .iter()
                    .map(|e| VerificationEvent::new_error(e).add_context("verif_schnorr_eb")),
            ),
    );
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
        assert!(
            result.is_ok(),
            "errors: {:?} \n failures: {:?}",
            result.errors(),
            result.failures()
        );
    }
}
