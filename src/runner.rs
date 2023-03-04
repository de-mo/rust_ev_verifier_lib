use crate::verification::{VerificationPeriod, VerificationsForPeriod};
use std::path::{Path, PathBuf};

pub struct Runner {
    path: PathBuf,
    verifications: VerificationsForPeriod,
}

impl Runner {
    fn new(path: &Path, period: VerificationPeriod) -> Runner {
        Runner {
            path: path.to_path_buf(),
            verifications: VerificationsForPeriod::new(period),
        }
    }

    fn run_all(&mut self) {
        for list in self.verifications.value_mut().values_mut() {
            for v in list.values_mut() {
                v.run();
            }
        }
    }
}
