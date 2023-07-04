use std::path::Path;

use anyhow::{anyhow, ensure};
use crypto_primitives::direct_trust::DirectTrust;
use rust_verifier_lib::{
    constants::{direct_trust_path, verification_list_path, SETUP_DIR_NAME, TALLY_DIR_NAME},
    verification::VerificationPeriod,
};

/// Check some elements at start of the application.
///
/// Must be caled by the application at the beginning. If error, then cannot continue
pub fn start_check() -> anyhow::Result<()> {
    ensure!(
        verification_list_path(None).exists(),
        format!(
            "List of verifications {:?} does not exist",
            verification_list_path(None).to_str()
        )
    );
    ensure!(
        direct_trust_path(None).is_dir(),
        format!(
            "Direct trust directory {:?} does not exist, or is not a directory",
            direct_trust_path(None).to_str()
        )
    );
    DirectTrust::new(&direct_trust_path(None))
        .map_err(|e| anyhow!("Cannot read keystore").context(e))?;
    Ok(())
}

pub fn is_directory_tally(path: &Path) -> anyhow::Result<bool> {
    ensure!(
        path.is_dir(),
        format!("Giveen directory {:?} does not exist", path)
    );
    ensure!(
        path.join(SETUP_DIR_NAME).is_dir(),
        format!(
            "The setup directory {:?} does not exist",
            path.join(SETUP_DIR_NAME)
        )
    );
    Ok(path.join(TALLY_DIR_NAME).is_dir())
}

pub fn check_verification_dir(period: &VerificationPeriod, path: &Path) -> anyhow::Result<()> {
    let is_tally = is_directory_tally(path)?;
    match period.is_tally() {
        true => match is_tally {
            true => Ok(()),
            false => Err(anyhow!(format!(
                "The tally directory {:?} does not exist",
                path.join(TALLY_DIR_NAME)
            ))),
        },
        false => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use super::{VerificationPeriod, *};
    use std::path::{Path, PathBuf};

    pub(crate) fn datasets_path() -> PathBuf {
        Path::new("..").join("datasets")
    }

    pub(crate) fn dataset_setup_path() -> PathBuf {
        datasets_path().join("dataset1-setup")
    }

    pub(crate) fn dataset_tally_path() -> PathBuf {
        datasets_path().join("dataset1-setup-tally")
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
