use crate::{
    file_structure::{CompletnessTestTrait, VerificationDirectory, VerificationDirectoryTrait},
    verification::{VerificationMetaDataList, VerificationPeriod},
    VerifierConfig,
};
use std::path::Path;

/// Check some elements before starting the verifications.
///
/// Must be called by the application at the beginning. If error, then cannot continue
pub fn start_check(config: &'static VerifierConfig) -> Result<(), String> {
    let md_list_check = VerificationMetaDataList::load(config.get_verification_list_str());
    if md_list_check.is_err() {
        return Err(format!(
            "List of verifications has an error: {}",
            md_list_check.unwrap_err()
        ));
    }
    config
        .keystore()
        .map_err(|e| format!("Cannot read keystore: {}", e))?;
    Ok(())
}

/// Check that the verification directory correct ist
pub fn check_verification_dir(period: &VerificationPeriod, path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("Given directory {:?} does not exist", path));
    };
    if !path.join(VerifierConfig::context_dir_name()).is_dir() {
        return Err(format!(
            "Directory {} does not exist in directory {:?}",
            VerifierConfig::context_dir_name(),
            path
        ));
    };
    let dir_name = match period {
        VerificationPeriod::Setup => VerifierConfig::setup_dir_name(),
        VerificationPeriod::Tally => VerifierConfig::tally_dir_name(),
    };
    if !path.join(dir_name).is_dir() {
        return Err(format!(
            "Directory {} does not exist in directory {:?}",
            dir_name, path
        ));
    };
    Ok(())
}

/// Check that the directory is complete
pub fn check_complete(
    period: &VerificationPeriod,
    dir: &VerificationDirectory,
) -> Result<(), String> {
    let context_complete = dir
        .context()
        .test_completness()
        .map_err(|e| e.to_string())?;
    if !context_complete.is_empty() {
        return Err(format!(
            "Context directory not complete: {}",
            context_complete.join(" / ")
        ));
    }
    let complete = match period {
        VerificationPeriod::Setup => dir
            .unwrap_setup()
            .test_completness()
            .map_err(|e| e.to_string())?,
        VerificationPeriod::Tally => dir
            .unwrap_tally()
            .test_completness()
            .map_err(|e| e.to_string())?,
    };
    if !complete.is_empty() {
        return Err(format!(
            "{} directory not complete: {}",
            period.as_ref(),
            complete.join(" / ")
        ));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::{VerificationPeriod, *};
    use crate::config::test::test_datasets_path;
    use std::path::Path;

    #[test]
    fn test_check_verification_dir() {
        assert!(check_verification_dir(&VerificationPeriod::Setup, Path::new("./toto")).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Tally, Path::new("./toto")).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Setup, Path::new(".")).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Tally, Path::new(".")).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Setup, &test_datasets_path()).is_ok());
        assert!(check_verification_dir(&VerificationPeriod::Tally, &test_datasets_path()).is_ok());
    }
}
