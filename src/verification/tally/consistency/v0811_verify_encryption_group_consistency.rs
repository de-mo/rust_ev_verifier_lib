use super::super::super::result::{VerificationEvent, VerificationResult};
use crate::{
    config::Config,
    file_structure::{
        context_directory::ContextDirectoryTrait,
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::elgamal::EncryptionParameters;

fn verify_encryption_group(
    eg: &EncryptionParameters,
    expected: &EncryptionParameters,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    if eg.p() != expected.p() {
        result.push(VerificationEvent::new_failure("p not equal"));
    }
    if eg.q() != expected.q() {
        result.push(VerificationEvent::new_failure("q not equal"));
    }
    if eg.g() != expected.g() {
        result.push(VerificationEvent::new_failure("g not equal"));
    }
    result
}

fn verify_encryption_group_for_tally_bb_dir<B: BBDirectoryTrait>(
    dir: &B,
    eg: &EncryptionParameters,
    result: &mut VerificationResult,
) {
    for (i, f) in dir.control_component_ballot_box_payload_iter() {
        match f {
            Ok(s) => result.append_with_context(
                &verify_encryption_group(&s.encryption_group, eg),
                format!("{}/control_component_ballot_box_payload.{}", dir.name(), i),
            ),
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_ballot_box_payload.{} has wrong format",
                dir.name(),
                i
            ))),
        }
    }

    for (i, f) in dir.control_component_shuffle_payload_iter() {
        match f {
            Ok(s) => result.append_with_context(
                &verify_encryption_group(&s.encryption_group, eg),
                format!("{}/control_component_shuffle_payload.{}", dir.name(), i),
            ),
            Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
                "{}/control_component_shuffle_payload.{} has wrong format",
                dir.name(),
                i
            ))),
        }
    }

    match dir.tally_component_shuffle_payload() {
        Ok(s) => result.append_with_context(
            &verify_encryption_group(&s.encryption_group, eg),
            format!("{}/tally_component_shuffle_payload", dir.name()),
        ),
        Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
            "{}/tally_component_shuffle_payload has wrong format",
            dir.name()
        ))),
    }

    match dir.tally_component_votes_payload() {
        Ok(s) => result.append_with_context(
            &verify_encryption_group(&s.encryption_group, eg),
            format!("{}/tally_component_votes_payload", dir.name()),
        ),
        Err(e) => result.push(VerificationEvent::new_error(&e).add_context(format!(
            "{}/tally_component_votes_payload has wrong format",
            dir.name()
        ))),
    }
}

pub(super) fn fn_verification<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static Config,
    result: &mut VerificationResult,
) {
    let config_dir = dir.context();
    let tally_dir = dir.unwrap_tally();
    let eg = match config_dir.election_event_context_payload() {
        Ok(p) => p.encryption_group,
        Err(e) => {
            result.push(
                VerificationEvent::new_error(&e)
                    .add_context("election_event_context_payload cannot be read"),
            );
            return;
        }
    };

    for bb in tally_dir.bb_directories().iter() {
        verify_encryption_group_for_tally_bb_dir(bb, &eg, result);
    }
}

#[cfg(test)]
mod test {
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;

    use super::*;
    use crate::config::test::{get_test_verifier_tally_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_verification(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_encryption_group() {
        let eg_expected = EncryptionParameters::from((
            &Integer::from(10usize),
            &Integer::from(15usize),
            &Integer::from(3usize),
        ));
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(10usize),
            &Integer::from(15usize),
            &Integer::from(3usize),
        ));
        result.append_with_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(result.is_ok());
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(11usize),
            &Integer::from(15usize),
            &Integer::from(3usize),
        ));
        result.append_with_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(!result.has_errors());
        assert_eq!(result.failures().len(), 1);
        let mut result = VerificationResult::new();
        let eg = EncryptionParameters::from((
            &Integer::from(11usize),
            &Integer::from(16usize),
            &Integer::from(4usize),
        ));
        result.append_with_context(&verify_encryption_group(&eg, &eg_expected), "toto");
        assert!(!result.has_errors());
        assert_eq!(result.failures().len(), 3)
    }
}
