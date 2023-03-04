pub mod error;
pub mod setup;
pub mod tally;
pub mod verification;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum VerificationCategory {
    Authenticity,
    Consistency,
    Completness,
    Integrity,
    Evidence,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    Stopped,
    Started,
    Success,
    Error,
    Failed,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum VerificationPeriod {
    Setup,
    Tally,
}

pub type VerificationList = HashMap<String, Box<verification::Verification>>;

pub type VerificationListCategory = HashMap<VerificationCategory, Box<VerificationList>>;

pub enum VerificationsForPeriod {
    Setup(Box<VerificationListCategory>),
    Tally(Box<VerificationListCategory>),
}

impl VerificationsForPeriod {
    pub fn new(period: VerificationPeriod) -> VerificationsForPeriod {
        match period {
            VerificationPeriod::Setup => {
                VerificationsForPeriod::Setup(Box::new(setup::get_verifications()))
            }
            VerificationPeriod::Tally => {
                VerificationsForPeriod::Tally(Box::new(tally::get_verifications()))
            }
        }
    }

    pub fn value(&self) -> &VerificationListCategory {
        match self {
            VerificationsForPeriod::Setup(vs) => vs.as_ref(),
            VerificationsForPeriod::Tally(vs) => vs.as_ref(),
        }
    }

    pub fn value_mut(&mut self) -> &mut VerificationListCategory {
        match self {
            VerificationsForPeriod::Setup(vs) => vs.as_mut(),
            VerificationsForPeriod::Tally(vs) => vs.as_mut(),
        }
    }
}
