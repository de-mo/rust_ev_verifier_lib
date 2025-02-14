use super::super::{
    meta_data::VerificationMetaDataList,
    result::{VerificationEvent, VerificationResult},
    suite::VerificationList,
    verifications::Verification,
};
use crate::{
    config::VerifierConfig,
    file_structure::{CompletnessTestTrait, VerificationDirectoryTrait},
    verification::VerificationError,
};

pub fn get_verifications<'a>(
    metadata_list: &'a VerificationMetaDataList,
    config: &'static VerifierConfig,
) -> Result<VerificationList<'a>, VerificationError> {
    Ok(VerificationList(vec![Verification::new(
        "01.01",
        "VerifySetupCompleteness",
        fn_0101_verify_setup_completeness,
        metadata_list,
        config,
    )?]))
}

fn fn_0101_verify_setup_completeness<D: VerificationDirectoryTrait>(
    dir: &D,
    _config: &'static VerifierConfig,
    result: &mut VerificationResult,
) {
    let context_dir: &<D as VerificationDirectoryTrait>::ContextDirType = dir.context();
    match context_dir.test_completness() {
        Ok(v) => result.append_failures_from_string_slice(&v),
        Err(e) => result.push(VerificationEvent::new_error(&e)),
    }
    let setup_dir = dir.unwrap_setup();
    match setup_dir.test_completness() {
        Ok(v) => result.append_failures_from_string_slice(&v),
        Err(e) => result.push(VerificationEvent::new_error(&e)),
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
        fn_0101_verify_setup_completeness(&dir, &CONFIG_TEST, &mut result);
        assert!(result.is_ok());
    }
}
