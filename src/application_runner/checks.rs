use std::path::Path;

use crate::{
    config::{Config as VerifierConfig, CONTEXT_DIR_NAME, SETUP_DIR_NAME, TALLY_DIR_NAME},
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

pub fn check_verification_dir(period: &VerificationPeriod, path: &Path) -> anyhow::Result<()> {
    ensure!(
        path.is_dir(),
        format!("Given directory {:?} does not exist", path)
    );
    ensure!(
        path.join(CONTEXT_DIR_NAME).is_dir(),
        format!(
            "Directory {} does not exist in directory {:?}",
            CONTEXT_DIR_NAME, path
        )
    );
    match period {
        VerificationPeriod::Setup => ensure!(
            path.join(SETUP_DIR_NAME).is_dir(),
            format!(
                "Directory {} does not exist in directory {:?}",
                SETUP_DIR_NAME, path
            )
        ),
        VerificationPeriod::Tally => ensure!(
            path.join(TALLY_DIR_NAME).is_dir(),
            format!(
                "Directory {} does not exist in directory {:?}",
                TALLY_DIR_NAME, path
            )
        ),
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
