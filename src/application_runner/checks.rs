use std::path::Path;

use crate::{
    config::Config as VerifierConfig,
    file_structure::{CompletnessTestTrait, VerificationDirectory, VerificationDirectoryTrait},
    verification::{meta_data::VerificationMetaDataList, VerificationPeriod},
};
use anyhow::{anyhow, ensure};

/// Check some elements at start of the application.
///
/// Must be called by the application at the beginning. If error, then cannot continue
pub fn start_check(config: &'static VerifierConfig) -> anyhow::Result<()> {
    let md_list_check = VerificationMetaDataList::load(config.get_verification_list_str());
    ensure!(
        md_list_check.is_ok(),
        format!(
            "List of verifications has an error: {}",
            md_list_check.unwrap_err()
        )
    );
    config
        .keystore()
        .map_err(|e| anyhow!("Cannot read keystore").context(e))?;
    Ok(())
}

/// Check that the verification directory correct ist
pub fn check_verification_dir(period: &VerificationPeriod, path: &Path) -> anyhow::Result<()> {
    ensure!(
        path.is_dir(),
        format!("Given directory {:?} does not exist", path)
    );
    ensure!(
        path.join(VerifierConfig::context_dir_name()).is_dir(),
        format!(
            "Directory {} does not exist in directory {:?}",
            VerifierConfig::context_dir_name(),
            path
        )
    );
    match period {
        VerificationPeriod::Setup => ensure!(
            path.join(VerifierConfig::setup_dir_name()).is_dir(),
            format!(
                "Directory {} does not exist in directory {:?}",
                VerifierConfig::setup_dir_name(),
                path
            )
        ),
        VerificationPeriod::Tally => ensure!(
            path.join(VerifierConfig::tally_dir_name()).is_dir(),
            format!(
                "Directory {} does not exist in directory {:?}",
                VerifierConfig::tally_dir_name(),
                path
            )
        ),
    }
    Ok(())
}

/// Check that the directory is complete
pub fn check_complete(
    period: &VerificationPeriod,
    dir: &VerificationDirectory,
) -> anyhow::Result<()> {
    let context_complete = dir.context().test_completness()?;
    if !context_complete.is_empty() {
        let mut e = anyhow!("Context directory not complete");
        for elt in context_complete.iter() {
            e = e.context(elt.clone());
        }
        return Err(e);
    }
    let complete = match period {
        VerificationPeriod::Setup => dir.unwrap_setup().test_completness()?,
        VerificationPeriod::Tally => dir.unwrap_tally().test_completness()?,
    };
    if !complete.is_empty() {
        let mut e = anyhow!(format!("directory {} not complete", period.to_string()));
        for elt in complete.iter() {
            e = e.context(elt.clone());
        }
        return Err(e);
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
