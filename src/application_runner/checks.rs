use std::path::Path;

use crate::{
    config::Config as VerifierConfig,
    verification::{meta_data::VerificationMetaDataList, VerificationPeriod},
};
use anyhow::{anyhow, ensure};
use rust_ev_crypto_primitives::direct_trust::DirectTrust;

/// Check some elements at start of the application.
///
/// Must be caled by the application at the beginning. If error, then cannot continue
pub fn start_check(config: &'static VerifierConfig) -> anyhow::Result<()> {
    let md_list_check = VerificationMetaDataList::load(config.get_verification_list_str());
    ensure!(
        md_list_check.is_ok(),
        format!(
            "List of verifications has an error: {}",
            md_list_check.unwrap_err()
        )
    );
    ensure!(
        config.direct_trust_dir_path().is_dir(),
        format!(
            "Direct trust directory {:?} does not exist, or is not a directory",
            config.direct_trust_dir_path().to_str()
        )
    );
    DirectTrust::new(&config.direct_trust_dir_path())
        .map_err(|e| anyhow!("Cannot read keystore").context(e))?;
    Ok(())
}

pub fn is_directory_tally(path: &Path) -> anyhow::Result<bool> {
    ensure!(
        path.is_dir(),
        format!("Giveen directory {:?} does not exist", path)
    );
    ensure!(
        path.join(VerifierConfig::setup_dir_name()).is_dir(),
        format!(
            "The setup directory {:?} does not exist",
            path.join(VerifierConfig::setup_dir_name())
        )
    );
    Ok(path.join(VerifierConfig::tally_dir_name()).is_dir())
}

pub fn check_verification_dir(period: &VerificationPeriod, path: &Path) -> anyhow::Result<()> {
    let is_tally = is_directory_tally(path)?;
    match period.is_tally() {
        true => match is_tally {
            true => Ok(()),
            false => Err(anyhow!(format!(
                "The tally directory {:?} does not exist",
                path.join(VerifierConfig::tally_dir_name())
            ))),
        },
        false => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use super::{VerificationPeriod, *};
    use crate::config::test::test_datasets_path;
    use std::path::{Path, PathBuf};

    pub(crate) fn dataset_setup_path() -> PathBuf {
        test_datasets_path().join("dataset1-setup")
    }

    pub(crate) fn dataset_tally_path() -> PathBuf {
        test_datasets_path().join("dataset1-setup-tally")
    }

    #[test]
    fn test_is_directory_tally() {
        assert!(is_directory_tally(Path::new("./toto")).is_err());
        assert!(is_directory_tally(Path::new(".")).is_err());
        assert!(!is_directory_tally(&dataset_setup_path()).unwrap());
        assert!(is_directory_tally(&dataset_tally_path()).unwrap());
    }

    #[test]
    fn test_check_verification_dir() {
        assert!(check_verification_dir(&VerificationPeriod::Setup, Path::new("./toto")).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Tally, Path::new("./toto")).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Setup, Path::new(".")).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Tally, Path::new(".")).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Setup, &dataset_setup_path()).is_ok());
        assert!(check_verification_dir(&VerificationPeriod::Tally, &dataset_setup_path()).is_err());
        assert!(check_verification_dir(&VerificationPeriod::Setup, &dataset_tally_path()).is_ok());
        assert!(check_verification_dir(&VerificationPeriod::Tally, &dataset_tally_path()).is_ok());
    }
}
