use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::VerifierConfig,
    data_structures::{
        context::control_component_public_keys_payload::ControlComponentPublicKeys,
        ControlComponentPublicKeysPayload,
    },
    file_structure::{context_directory::ContextDirectoryTrait, VerificationDirectoryTrait},
};

fn validate_cc_ccr_enc_pk<S: ContextDirectoryTrait>(
    context_dir: &S,
    setup: &ControlComponentPublicKeys,
    node_id: usize,
    result: &mut VerificationResult,
) {
    let f = context_dir
        .control_component_public_keys_payload_group()
        .get_file_with_number(node_id);
    let cc_pk = match f
        .decode_verifier_data::<ControlComponentPublicKeysPayload>()
        .map(Box::new)
    {
        Ok(d) => d.control_component_public_keys,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context(format!("Cannot read data from file {}", f.path_to_str())),
            );
            return;
        }
    };
    if setup.ccrj_choice_return_codes_encryption_public_key.len()
        != cc_pk.ccrj_choice_return_codes_encryption_public_key.len()
    {
        result.push(VerificationEvent::new_failure(&format!("The length of CCR Choice Return Codes encryption public keys for control component {} are identical from both sources", node_id)));
    } else if setup.ccrj_choice_return_codes_encryption_public_key
        != cc_pk.ccrj_choice_return_codes_encryption_public_key
    {
        result.push(VerificationEvent::new_failure(&format!("The CCR Choice Return Codes encryption public keys for control component {} are identical from both sources", node_id)));
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let sc_pk = match context_dir.setup_component_public_keys_payload() {
        Ok(o) => o,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("Cannot extract setup_component_public_keys_payload"),
            );
            return;
        }
    };
    for node in sc_pk
        .setup_component_public_keys
        .combined_control_component_public_keys
    {
        validate_cc_ccr_enc_pk(context_dir, &node, node.node_id, result)
    }
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
