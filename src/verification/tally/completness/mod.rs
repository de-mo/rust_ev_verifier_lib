use crate::{
    config::VerifierConfig,
    file_structure::{CompletnessTestTrait, VerificationDirectoryTrait},
    verification::VerificationError,
};

use super::super::{
    meta_data::VerificationMetaDataList,
    result::{VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
};
pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static VerifierConfig,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![Verification::new(
        "06.01",
        "VerifyTallyCompleteness",
        fn_0601_verify_tally_completeness,
        metadata_list,
        config,
    )?]))
}

fn fn_0601_verify_tally_completeness<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir = dir.context();
    match context_dir.test_completness() {
        Ok(v) => result.append_failures_from_string_slice(&v),
        Err(e) => result.push(VerificationEvent::new_error(&e)),
    }
    let tally_dir = dir.unwrap_tally();
    match tally_dir.test_completness() {
        Ok(v) => result.append_failures_from_string_slice(&v),
        Err(e) => result.push(VerificationEvent::new_error(&e)),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_test_verifier_tally_dir, CONFIG_TEST};

    #[test]
    fn test_ok() {
        let dir = get_test_verifier_tally_dir();
        let mut result = VerificationResult::new();
        fn_0601_verify_tally_completeness(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
