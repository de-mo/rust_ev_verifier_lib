pub mod constants;
mod data_structures;
mod file_structure;
pub mod runner;
pub mod verification;

use std::path::Path;

use anyhow::{anyhow, ensure};
use constants::{direct_trust_path, SETUP_DIR_NAME};
use crypto_primitives::direct_trust::DirectTrust;
use verification::VerificationPeriod;

use crate::constants::{verification_list_path, TALLY_DIR_NAME};

/// Check some elements at start of the application.
///
/// Must be caled by the application at the beginning. If error, then cannot continue
pub fn start_check() -> anyhow::Result<()> {
    ensure!(
        verification_list_path().exists(),
        format!(
            "List of verifications {:?} does not exist",
            verification_list_path().to_str()
        )
    );
    ensure!(
        direct_trust_path().is_dir(),
        format!(
            "Direct trust directory {:?} does not exist, or is not a directory",
            direct_trust_path().to_str()
        )
    );
    DirectTrust::new(&direct_trust_path())
        .map_err(|e| anyhow!("Cannot read keystore").context(e))?;
    Ok(())
}

pub fn check_verification_dir(period: &VerificationPeriod, path: &Path) -> anyhow::Result<()> {
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
    if period.is_tally() {
        ensure!(
            path.join(TALLY_DIR_NAME).is_dir(),
            format!(
                "The setup directory {:?} does not exist",
                path.join(TALLY_DIR_NAME)
            )
        );
    }
    Ok(())
}
