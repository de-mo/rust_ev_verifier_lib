// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use tracing::trace;

use super::super::{
    result::{VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
    verify_signature_for_object,
};
use crate::{
    config::VerifierConfig,
    file_structure::{
        context_directory::{ContextDirectoryTrait, ContextVCSDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::{meta_data::VerificationMetaDataList, VerificationError, VerificationErrorImpl},
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static VerifierConfig,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![
        Verification::new(
            "02.01",
            "VerifySignatureCantonConfig",
            fn_0201_verify_signature_canton_config,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureCantonConfig",
            source: Box::new(e),
        })?,
        Verification::new(
            "02.02",
            "VerifySignatureSetupComponentPublicKeys",
            fn_0202_verify_signature_setup_component_public_keys,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureSetupComponentPublicKeys",
            source: Box::new(e),
        })?,
        Verification::new(
            "02.03",
            "VerifySignatureControlComponentPublicKeys",
            fn_0203_verify_signature_control_component_public_keys,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureControlComponentPublicKeys",
            source: Box::new(e),
        })?,
        Verification::new(
            "02.04",
            "VerifySignatureSetupComponentTallyData",
            fn_0204_verify_signature_setup_component_tally_data,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureSetupComponentTallyData",
            source: Box::new(e),
        })?,
        Verification::new(
            "02.05",
            "VerifySignatureElectionEventContext",
            fn_0205_verify_signature_election_event_context,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureElectionEventContext",
            source: Box::new(e),
        })?,
    ]))
}

fn fn_0201_verify_signature_canton_config<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let ee_config = match context_dir.election_event_configuration() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e)
                    .add_context(format!("{} cannot be read", "election_event_configuration")),
            );
            return;
        }
    };
    result.append_with_context(
        &verify_signature_for_object(ee_config.as_ref(), config),
        "election_event_configuration",
    )
}

fn fn_0202_verify_signature_setup_component_public_keys<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let payload = match context_dir.setup_component_public_keys_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{} cannot be read",
                    "setup_component_public_keys_payload"
                )),
            );
            return;
        }
    };
    result.append_with_context(
        &verify_signature_for_object(payload.as_ref(), config),
        "setup_component_public_keys_payload",
    )
}

fn fn_0203_verify_signature_control_component_public_keys<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    for (i, cc) in context_dir.control_component_public_keys_payload_iter() {
        trace!("Verification 2.03 for cc {}", i);
        match cc {
            Ok(cc) => result.append_with_context(
                &verify_signature_for_object(cc.as_ref(), config),
                format!("control_component_public_keys_payload_{i}"),
            ),
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!("control_component_public_keys_payload_{i} cannot be read"),
            )),
        }
    }
}

fn fn_0204_verify_signature_setup_component_tally_data<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    for d in context_dir.vcs_directories() {
        trace!("Verification 2.04 for vcs_dir {}", d.name());
        match d.setup_component_tally_data_payload() {
            Ok(p) => result.append_with_context(
                &verify_signature_for_object(p.as_ref(), config),
                format!("{}/setup_component_tally_data_payload.json", d.name(),),
            ),
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(
                format!("{}/setup_component_tally_data_payload.json", d.name(),),
            )),
        }
    }
}

fn fn_0205_verify_signature_election_event_context<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    let rp = match context_dir.election_event_context_payload() {
        Ok(p) => p,
        Err(e) => {
            result.push(
                VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{} cannot be read",
                    "election_event_context_payload"
                )),
            );
            return;
        }
    };
    result.append_with_context(
        &verify_signature_for_object(rp.as_ref(), config),
        "election_event_context_payload",
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_setup_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_0201() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0201_verify_signature_canton_config(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{e:?}");
            }
            for f in result.failures() {
                println!("{f:?}");
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_0202() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0202_verify_signature_setup_component_public_keys(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_0203() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0203_verify_signature_control_component_public_keys(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_0204() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0204_verify_signature_setup_component_tally_data(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_0205() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0205_verify_signature_election_event_context(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
