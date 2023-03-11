use std::path::Path;

use crate::verification::VerificationPeriod;

use super::{setup_directory::SetupDirectory, tally_directory::TallyDirectory};

pub enum VerificationDirectory {
    Setup(SetupDirectory),
    Tally(TallyDirectory),
}

impl VerificationDirectory {
    pub fn new(period: VerificationPeriod, location: &Path) -> VerificationDirectory {
        match period {
            VerificationPeriod::Setup => {
                VerificationDirectory::Setup(SetupDirectory::new(location))
            }
            VerificationPeriod::Tally => {
                VerificationDirectory::Tally(TallyDirectory::new(location))
            }
        }
    }

    pub fn get_setup(&self) -> Option<&SetupDirectory> {
        match self {
            VerificationDirectory::Setup(d) => Some(d),
            VerificationDirectory::Tally(_) => None,
        }
    }

    pub fn get_tally(&self) -> Option<&TallyDirectory> {
        match self {
            VerificationDirectory::Setup(_) => None,
            VerificationDirectory::Tally(d) => Some(d),
        }
    }
}
