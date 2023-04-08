use crate::{
    file_structure::VerificationDirectory,
    verification::{VerificationListTrait, VerificationPeriod, VerificationsForPeriod},
};
use log::{info, warn};
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

pub struct Runner {
    path: PathBuf,
    period: VerificationPeriod,
    verifications: VerificationsForPeriod,
}

impl Runner {
    pub fn new(path: &Path, period: VerificationPeriod) -> Runner {
        Runner {
            path: path.to_path_buf(),
            period,
            verifications: VerificationsForPeriod::new(period),
        }
    }

    pub fn run_all(&mut self, exclusion: &Vec<&String>) {
        let start_time = SystemTime::now();
        info!(
            "Start all verifications ({} verifications; {} excluded)",
            self.verifications.len_with_exclusion(exclusion),
            self.verifications.len_excluded(exclusion)
        );
        let directory = VerificationDirectory::new(self.period, &self.path);
        for v in self.verifications.value_mut().iter_mut() {
            if !exclusion.contains(&&v.meta_data.id) {
                v.run(&directory);
            } else {
                warn!(
                    "Verification {} ({}) skipped",
                    v.meta_data.name, v.meta_data.id
                )
            }
        }
        let duration = start_time.elapsed().unwrap();
        info!(
            "{} verifications run (duration: {}s)",
            self.verifications.len_with_exclusion(exclusion),
            duration.as_secs_f32()
        );
    }
}
