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

use super::super::{
    result::VerificationEvent, suite::VerificationList, verifications::Verification,
};
use crate::{
    config::VerifierConfig,
    file_structure::{
        tally_directory::{BBDirectoryTrait, TallyDirectoryTrait},
        VerificationDirectoryTrait,
    },
    verification::{
        meta_data::VerificationMetaDataList, result::VerificationResult,
        verification_unimplemented, verify_signature_for_object, VerificationError,
        VerificationErrorImpl,
    },
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static VerifierConfig,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![
        Verification::new(
            "07.01",
            "VerifySignatureControlComponentBallotBox",
            fn_0701_verify_signature_control_component_ballot_box,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureControlComponentBallotBox",
            source: Box::new(e),
        })?,
        Verification::new(
            "07.02",
            "VerifySignatureControlComponentShuffle",
            fn_0702_verify_verify_signature_control_component_shuffle,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureControlComponentShuffle",
            source: Box::new(e),
        })?,
        Verification::new(
            "07.03",
            "VerifySignatureTallyComponentShuffle",
            fn_0703_verify_signature_tally_component_shuffle,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureTallyComponentShuffle",
            source: Box::new(e),
        })?,
        Verification::new(
            "07.04",
            "VerifySignatureTallyComponentVotes",
            fn_0704_verify_signature_tally_component_votes,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureTallyComponentVotes",
            source: Box::new(e),
        })?,
        Verification::new(
            "07.05",
            "VerifySignatureTallyComponentDecrypt",
            verification_unimplemented,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureTallyComponentDecrypt",
            source: Box::new(e),
        })?,
        Verification::new(
            "07.06",
            "VerifySignatureTallyComponentEch0222",
            verification_unimplemented,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureTallyComponentEch0222",
            source: Box::new(e),
        })?,
        Verification::new(
            "07.07",
            "VerifySignatureTallyComponentEch0110",
            verification_unimplemented,
            metadata_list,
            config,
        )
        .map_err(|e| VerificationErrorImpl::GetVerification {
            name: "VerifySignatureTallyComponentEch0110",
            source: Box::new(e),
        })?,
    ]))
}

fn fn_0701_verify_signature_control_component_ballot_box<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    for bb_d in tally_dir.bb_directories().iter() {
        for (i, f) in bb_d.control_component_ballot_box_payload_iter() {
            match f {
                Ok(d) => result.append_with_context(
                    &verify_signature_for_object(d.as_ref(), config),
                    format!(
                        "{}/control_component_ballot_box_payload_{}.json",
                        bb_d.name(),
                        i
                    ),
                ),
                Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{}/control_component_ballot_box_payload_{}.json",
                    bb_d.name(),
                    i
                ))),
            }
        }
    }
}

fn fn_0702_verify_verify_signature_control_component_shuffle<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    for bb_d in tally_dir.bb_directories().iter() {
        for (i, f) in bb_d.control_component_shuffle_payload_iter() {
            match f {
                Ok(d) => result.append_with_context(
                    &verify_signature_for_object(d.as_ref(), config),
                    format!(
                        "{}/control_component_shuffle_payload_{}.json",
                        bb_d.name(),
                        i
                    ),
                ),
                Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                    "{}/control_component_shuffle_payload_{}.json",
                    bb_d.name(),
                    i
                ))),
            }
        }
    }
}

fn fn_0703_verify_signature_tally_component_shuffle<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    for bb_d in tally_dir.bb_directories().iter() {
        match bb_d.tally_component_shuffle_payload() {
            Ok(d) => result.append_with_context(
                &verify_signature_for_object(d.as_ref(), config),
                format!("{}/tally_component_shuffle_payload.json", bb_d.name(),),
            ),
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/tally_component_shuffle_payload.json",
                bb_d.name(),
            ))),
        }
    }
}

fn fn_0704_verify_signature_tally_component_votes<D: VerificationDirectoryTrait>(
    dir: &D,
    config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let tally_dir = dir.unwrap_tally();
    for bb_d in tally_dir.bb_directories().iter() {
        match bb_d.tally_component_votes_payload() {
            Ok(d) => result.append_with_context(
                &verify_signature_for_object(d.as_ref(), config),
                format!("{}/tally_component_votes_payload.json", bb_d.name(),),
            ),
            Err(e) => result.push(VerificationEvent::new_error_from_error(&e).add_context(format!(
                "{}/tally_component_votes_payload.json",
                bb_d.name(),
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_tally_dir as get_verifier_dir, CONFIG_TEST};

    #[test]
    fn test_0701() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0701_verify_signature_control_component_ballot_box(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_0702() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0702_verify_verify_signature_control_component_shuffle(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_0703() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0703_verify_signature_tally_component_shuffle(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_0704() {
        let dir = get_verifier_dir();
        let mut result = VerificationResult::new();
        fn_0704_verify_signature_tally_component_votes(&dir, &CONFIG_TEST, &mut result);
        if !result.is_ok() {
            for e in result.errors() {
                println!("{:?}", e);
            }
            for f in result.failures() {
                println!("{:?}", f);
            }
        }
        assert!(result.is_ok());
    }
}
