use crate::{
    file_structure::VerificationDirectory,
    verification::{VerificationPeriod, VerificationsForPeriod},
};
use std::path::{Path, PathBuf};

pub struct Runner {
    path: PathBuf,
    period: VerificationPeriod,
    verifications: VerificationsForPeriod,
}

impl Runner {
    fn new(path: &Path, period: VerificationPeriod) -> Runner {
        Runner {
            path: path.to_path_buf(),
            period,
            verifications: VerificationsForPeriod::new(period),
        }
    }

    fn run_all(&mut self) {
        let directory = VerificationDirectory::new(self.period, &self.path);
        for v in self.verifications.value_mut().iter_mut() {
            v.run(&directory);
        }
    }
}
